use crate::recipe::{Compose, Copy, Lang, Recipe, Rule, Slice};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// bump when the same recipe and download would now give different files, so every cache rebuilds
const CONVERTER_VERSION: u32 = 1;
const STAMP_FILE: &str = ".stamp";
const LANG_OUT: &str = "lang/en_US.lang";
const ENTRY_LIMIT: u64 = 256 << 20;
const RGBA: usize = 4;

pub enum Outcome
{
    Converted(usize),
    Cached,
}

// cache holds assets/<modid>/... ready to mount as a modern pack
pub fn install(recipeText: &str, modId: &str, input: &Path, cache: &Path) -> Result<Outcome, String>
{
    let recipe: Recipe = Recipe::parse(recipeText)?;
    let stamped: String = std::fs::read_to_string(cache.join(STAMP_FILE)).unwrap_or_default();
    let rules: String = hash(&[CONVERTER_VERSION.to_string().as_str(), recipeText]);
    let empty: bool = std::fs::read_dir(input).map_or(true, |mut entries| entries.next().is_none());

    // the player may delete the downloads once a cache from this recipe exists
    if empty && stamped.lines().next() == Some(rules.as_str())
    {
        return Ok(Outcome::Cached);
    }

    let downloads: Vec<(PathBuf, String)> = find(&recipe, input)?;
    let hashes: Vec<&str> = downloads.iter().map(|(_, hash)| hash.as_str()).collect();
    let stamp: String = format!("{rules}\n{}\n", hash(&hashes));

    if stamped == stamp
    {
        return Ok(Outcome::Cached);
    }

    let temp: PathBuf = cache.with_extension("tmp");
    remove(&temp)?;

    let converted: Result<usize, String> = convert(&recipe, &downloads, &temp.join("assets").join(modId))
        .and_then(|count| write(&temp.join(STAMP_FILE), stamp.as_bytes()).map(|()| count));

    let count: usize = match converted
    {
        Ok(count) => count,
        Err(reason) =>
        {
            remove(&temp)?;
            return Err(reason);
        }
    };

    // the stamp only appears with a finished cache so a crash mid convert retries next launch
    remove(cache)?;
    std::fs::rename(&temp, cache).map_err(|error| format!("{}: {error}", cache.display()))?;
    Ok(Outcome::Converted(count))
}

// one file per download, matched by hash, else by name when the recipe lists no hashes
fn find(recipe: &Recipe, input: &Path) -> Result<Vec<(PathBuf, String)>, String>
{
    let mut files: Vec<PathBuf> = match std::fs::read_dir(input)
    {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).filter(|path| path.is_file()).collect(),
        Err(_) => Vec::new(),
    };
    files.sort();

    let mut hashes: Vec<String> = Vec::new();
    for file in &files
    {
        hashes.push(hashFile(file)?);
    }

    let mut found: Vec<(PathBuf, String)> = Vec::new();
    for download in &recipe.download
    {
        let named = |file: &PathBuf| file.file_name().is_some_and(|name| name == download.name.as_str());
        let index: Option<usize> = if download.sha256.is_empty()
        {
            files.iter().position(named)
        }
        else
        {
            hashes.iter().position(|hash| download.sha256.iter().any(|wanted| wanted.eq_ignore_ascii_case(hash)))
        };

        let Some(index) = index
        else
        {
            let wrong: bool = files.iter().any(named);
            let reason: &str = if wrong
            {
                "is an unknown version"
            }
            else
            {
                "is missing"
            };
            let from: String = if download.url.is_empty()
            {
                String::new()
            }
            else
            {
                format!(", get it from {}", download.url)
            };
            return Err(format!("{} {reason}, put it in {}{from}", download.name, input.display()));
        };

        found.push((files[index].clone(), hashes[index].clone()));
    }

    Ok(found)
}

fn hash(parts: &[&str]) -> String
{
    let mut hasher: Sha256 = Sha256::new();
    for part in parts
    {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }

    hex(&hasher.finalize())
}

fn convert(recipe: &Recipe, downloads: &[(PathBuf, String)], out: &Path) -> Result<usize, String>
{
    let mut archives: Vec<Archive> = Vec::new();
    for (path, _) in downloads
    {
        archives.push(Archive::open(path)?);
    }

    let mut count: usize = 0;

    for copy in &recipe.copy
    {
        count += copyFiles(copy, &mut archives[recipe.download(copy.source.as_deref())?], out)?;
    }

    for slice in &recipe.slice
    {
        count += cut(slice, &mut archives[recipe.download(slice.source.as_deref())?], out)?;
    }

    for compose in &recipe.compose
    {
        build(compose, &mut archives[recipe.download(compose.source.as_deref())?], out)?;
        count += 1;
    }

    let mut lines: Vec<String> = Vec::new();
    for lang in &recipe.lang
    {
        lines.extend(translate(lang, &mut archives[recipe.download(lang.source.as_deref())?])?);
    }

    if !lines.is_empty()
    {
        write(&out.join(LANG_OUT), format!("{}\n", lines.join("\n")).as_bytes())?;
        count += 1;
    }

    Ok(count)
}

fn copyFiles(copy: &Copy, archive: &mut Archive, out: &Path) -> Result<usize, String>
{
    if !copy.from.ends_with('/')
    {
        write(&out.join(&copy.to), &archive.read(&copy.from)?)?;
        return Ok(1);
    }

    let names: Vec<String> = archive.list(&copy.from)?;
    if names.is_empty()
    {
        return Err(format!("{} has no files below {}", archive.name(), copy.from));
    }

    for name in &names
    {
        let target: String = format!("{}{}", copy.to, &name[copy.from.len()..]);
        if !crate::recipe::inside(&target)
        {
            return Err(format!("{} entry {name} leaves the mod's assets folder", archive.name()));
        }

        write(&out.join(target), &archive.read(name)?)?;
    }

    Ok(names.len())
}

fn cut(slice: &Slice, archive: &mut Archive, out: &Path) -> Result<usize, String>
{
    let sheet: Image =
        Image::decode(&archive.read(&slice.from)?).map_err(|error| format!("{}: {error}", slice.from))?;
    let size: usize = slice.tile as usize;
    let columns: usize = sheet.m_width / size;

    for (slot, name) in &slice.tiles
    {
        let slot: usize = slot.parse().map_err(|_| format!("slice {} tile {slot} is not a number", slice.from))?;
        let (column, row): (usize, usize) = (slot % columns.max(1), slot / columns.max(1));

        if (row + 1) * size > sheet.m_height || columns == 0
        {
            return Err(format!("{} has no tile {slot} at size {size}", slice.from));
        }

        let tile: Vec<u8> = sheet.crop(column * size, row * size, size, size);
        encode(&out.join(format!("{}{name}.png", slice.to)), &tile, size, size)?;
    }

    Ok(slice.tiles.len())
}

fn build(compose: &Compose, archive: &mut Archive, out: &Path) -> Result<(), String>
{
    let [width, height]: [u32; 2] = compose.canvas;
    let mut canvas: Image = Image::filled(width as usize, height as usize, compose.color()?);

    if let Some(from) = &compose.from
    {
        let image: Image = Image::decode(&archive.read(from)?).map_err(|error| format!("{from}: {error}"))?;
        for part in &compose.parts
        {
            let [left, top, partWidth, partHeight]: [usize; 4] = part.rect.map(|value| value as usize);
            if left + partWidth > image.m_width || top + partHeight > image.m_height
            {
                return Err(format!("{from} has no region {:?}", part.rect));
            }

            let pixels: Vec<u8> = image.crop(left, top, partWidth, partHeight);
            canvas.paste(&pixels, partWidth, part.at.map(|value| value as usize));
        }
    }

    let [outWidth, outHeight]: [u32; 2] = compose.size.unwrap_or(compose.canvas);
    let scaled: Image = canvas.scale(outWidth as usize, outHeight as usize);
    encode(&out.join(&compose.to), &scaled.m_pixels, scaled.m_width, scaled.m_height)
}

fn translate(lang: &Lang, archive: &mut Archive) -> Result<Vec<String>, String>
{
    let text: String =
        String::from_utf8(archive.read(&lang.from)?).map_err(|_| format!("{} is not UTF-8", lang.from))?;
    let pairs: Vec<(String, String)> = if lang.from.ends_with(".json")
    {
        let table: BTreeMap<String, String> =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", lang.from))?;
        table.into_iter().collect()
    }
    else
    {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.split_once('='))
            .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
            .collect()
    };

    Ok(pairs
        .iter()
        .filter_map(|(key, value)|
        {
            lang.rules.iter().find_map(|rule| rename(rule, key)).map(|key| format!("{key}={value}"))
        })
        .collect())
}

fn rename(rule: &Rule, key: &str) -> Option<String>
{
    let rest: &str = key.strip_prefix(rule.from.as_str())?;
    let rest: &str = if rule.cut.is_empty()
    {
        rest
    }
    else
    {
        rest.strip_suffix(rule.cut.as_str()).unwrap_or(rest)
    };
    Some(format!("{}{rest}{}", rule.to, rule.add))
}

struct Archive
{
    m_path: PathBuf,
    m_zip: ZipArchive<File>,
}

impl Archive
{
    fn open(path: &Path) -> Result<Self, String>
    {
        let file: File = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let zip: ZipArchive<File> =
            ZipArchive::new(file).map_err(|error| format!("{} is not a zip: {error}", path.display()))?;
        Ok(Self { m_path: path.to_path_buf(), m_zip: zip })
    }

    fn name(&self) -> String
    {
        self.m_path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    }

    fn read(&mut self, entry: &str) -> Result<Vec<u8>, String>
    {
        let name: String = self.name();
        let file = self.m_zip.by_name(entry).map_err(|_| format!("{entry} is not in {name}"))?;
        let mut bytes: Vec<u8> = Vec::new();
        // the declared size is not trusted so a forged entry cannot inflate without bound
        file.take(ENTRY_LIMIT + 1).read_to_end(&mut bytes).map_err(|error| format!("{entry} in {name}: {error}"))?;

        if bytes.len() as u64 > ENTRY_LIMIT
        {
            return Err(format!("{entry} in {name} is over the {ENTRY_LIMIT} byte limit"));
        }

        Ok(bytes)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, String>
    {
        let mut names: Vec<String> = Vec::new();
        for name in self.m_zip.file_names()
        {
            let name: String = name.map_err(|error| format!("{}: {error}", self.name()))?.into_owned();
            if name.starts_with(prefix) && !name.ends_with('/')
            {
                names.push(name);
            }
        }

        names.sort();
        Ok(names)
    }
}

struct Image
{
    m_width: usize,
    m_height: usize,
    m_pixels: Vec<u8>,
}

impl Image
{
    // any png layout widened to 8 bit rgba
    fn decode(bytes: &[u8]) -> Result<Self, String>
    {
        let mut decoder: png::Decoder<&[u8]> = png::Decoder::new(bytes);
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);

        let mut reader: png::Reader<&[u8]> = decoder.read_info().map_err(|error| error.to_string())?;
        let mut buffer: Vec<u8> = vec![0; reader.output_buffer_size()];
        let info: png::OutputInfo = reader.next_frame(&mut buffer).map_err(|error| error.to_string())?;
        buffer.truncate(info.buffer_size());

        let pixels: Vec<u8> = match info.color_type
        {
            png::ColorType::Rgba => buffer,
            png::ColorType::Rgb =>
            {
                buffer.as_chunks::<3>().0.iter().flat_map(|&[red, green, blue]| [red, green, blue, u8::MAX]).collect()
            }
            png::ColorType::GrayscaleAlpha =>
            {
                buffer.as_chunks::<2>().0.iter().flat_map(|&[gray, alpha]| [gray, gray, gray, alpha]).collect()
            }
            png::ColorType::Grayscale => buffer.iter().flat_map(|&gray| [gray, gray, gray, u8::MAX]).collect(),
            png::ColorType::Indexed => return Err("indexed png was not expanded".to_owned()),
        };

        Ok(Self { m_width: info.width as usize, m_height: info.height as usize, m_pixels: pixels })
    }

    fn filled(width: usize, height: usize, color: [u8; 4]) -> Self
    {
        Self { m_width: width, m_height: height, m_pixels: color.repeat(width * height) }
    }

    fn crop(&self, left: usize, top: usize, width: usize, height: usize) -> Vec<u8>
    {
        let mut tile: Vec<u8> = Vec::with_capacity(width * height * RGBA);
        for y in top..top + height
        {
            let start: usize = (y * self.m_width + left) * RGBA;
            tile.extend_from_slice(&self.m_pixels[start..start + width * RGBA]);
        }

        tile
    }

    // pixels replace what is below them alpha included
    fn paste(&mut self, pixels: &[u8], width: usize, at: [usize; 2])
    {
        let [x, y]: [usize; 2] = at;
        for (row, line) in pixels.chunks_exact(width * RGBA).enumerate()
        {
            let start: usize = ((y + row) * self.m_width + x) * RGBA;
            self.m_pixels[start..start + line.len()].copy_from_slice(line);
        }
    }

    // nearest pixel from each target pixel's center like PIL resize with NEAREST
    fn scale(self, width: usize, height: usize) -> Self
    {
        if width == self.m_width && height == self.m_height
        {
            return self;
        }

        let mut pixels: Vec<u8> = Vec::with_capacity(width * height * RGBA);
        for y in 0..height
        {
            let sourceY: usize = (y * 2 + 1) * self.m_height / (height * 2);
            for x in 0..width
            {
                let sourceX: usize = (x * 2 + 1) * self.m_width / (width * 2);
                let start: usize = (sourceY * self.m_width + sourceX) * RGBA;
                pixels.extend_from_slice(&self.m_pixels[start..start + RGBA]);
            }
        }

        Self { m_width: width, m_height: height, m_pixels: pixels }
    }
}

fn encode(path: &Path, pixels: &[u8], width: usize, height: usize) -> Result<(), String>
{
    create(path)?;
    let file: File = File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder: png::Encoder<BufWriter<File>> =
        png::Encoder::new(BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer: png::Writer<BufWriter<File>> =
        encoder.write_header().map_err(|error| format!("{}: {error}", path.display()))?;
    writer.write_image_data(pixels).map_err(|error| format!("{}: {error}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String>
{
    create(path)?;
    std::fs::write(path, bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn create(path: &Path) -> Result<(), String>
{
    let Some(parent) = path.parent()
    else
    {
        return Ok(());
    };

    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))
}

fn remove(path: &Path) -> Result<(), String>
{
    if !path.exists()
    {
        return Ok(());
    }

    std::fs::remove_dir_all(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn hashFile(path: &Path) -> Result<String, String>
{
    let mut file: File = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut hasher: Sha256 = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|byte| format!("{byte:02x}")).collect() }
