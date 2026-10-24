# @j3lte/unidata

[![JSR](https://jsr.io/badges/@j3lte/unidata)](https://jsr.io/@j3lte/unidata)
[![GitHub Release](https://img.shields.io/github/v/release/j3lte/unidata)](https://github.com/j3lte/unidata/releases/latest)
[![NPM Version](https://img.shields.io/npm/v/unidata16)](https://www.npmjs.com/package/unidata16)
[![NPM Downloads](https://img.shields.io/npm/dm/unidata16)](https://www.npmjs.com/package/unidata16)
[![License](https://img.shields.io/github/license/j3lte/unidata)](https://github.com/j3lte/unidata/blob/main/LICENSE)

A comprehensive Unicode data library for Deno and Node.js, providing access to the latest Unicode 17.0.0 character sets and metadata.

## ✨ Features

- **Latest Unicode Support**: Based on [Unicode Standard 17.0.0](https://www.unicode.org/versions/Unicode17.0.0/)
- **Complete Character Sets**: Access to all 400+ Unicode blocks
- **Rich Metadata**: Character codes, names, categories, and properties
- **TypeScript Support**: Full type definitions included
- **Cross-Platform**: Works with both Deno and Node.js
- **Lightweight**: Efficient data structures and tree-shaking support

## 🚀 Installation

### Deno (Recommended)

```bash
deno add @j3lte/unidata
```

### NPM

```bash
npm install unidata16
```

## 📦 Documentation

- [Deno Docs](https://jsr.io/@j3lte/unidata/doc)
- [NPM Package](https://www.npmjs.com/package/unidata16)

## 📖 Usage

### Basic Character Retrieval

```typescript
import { CharacterSetType, getCharacters } from "@j3lte/unidata"; // or "unidata16" if you installed via NPM

// Get all characters from a specific Unicode block
const emoji = getCharacters(CharacterSetType.Emoticons);
console.log(emoji.length); // Number of emoticon characters

// Get characters from multiple blocks
const symbols = getCharacters([
  CharacterSetType.Mathematical_Operators,
  CharacterSetType.Arrows,
  CharacterSetType.Currency_Symbols
]);
```

### Character Set Information

```typescript
import { CharacterSetType, getCharacterBaseSet, getCharacterSet } from "@j3lte/unidata"; // or "unidata16" if you installed via NPM

// Get basic information about a character set
const basicInfo = getCharacterBaseSet(CharacterSetType.Basic_Latin);
console.log(basicInfo);
// {
//   blockName: "Basic Latin",
//   startCode: 0x0020,
//   endCode: 0x007F
// }

// Get complete character set with all character data
const fullSet = getCharacterSet(CharacterSetType.Greek_and_Coptic);
console.log(fullSet.characters[0]);
// {
//   code: 0x0370,
//   name: "GREEK CAPITAL LETTER HETA",
//   cat: "Lu",
//   comb: undefined,
//   bidi: "L",
//   ...
// }
```

### Working with Character Data

```typescript
import { CharacterSetType, getCharacters } from "@j3lte/unidata"; // or "unidata16" if you installed via NPM

// Find characters by properties
const digits = getCharacters(CharacterSetType.Superscripts_and_Subscripts)
  .filter(char => char.num !== undefined);

// Get all currency symbols
const currencies = getCharacters(CharacterSetType.Currency_Symbols)
  .map(char => ({
    symbol: String.fromCodePoint(char.code),
    name: char.name,
    code: char.code
  }));

// Find characters with specific categories
const letters = getCharacters(CharacterSetType.Latin_Extended_A)
  .filter(char => char.cat.startsWith('L')); // All letter categories
```

### Advanced Usage

```typescript
import { CharacterSetType, getCharacters, UNICODE_VERSION } from "@j3lte/unidata"; // or "unidata16" if you installed via NPM

// Get all characters from all blocks
const allCharacters = getCharacters(null);
console.log(`Total characters in Unicode ${UNICODE_VERSION}: ${allCharacters.length}`);

// Create a character lookup by code
const charMap = new Map(
  getCharacters(CharacterSetType.Basic_Latin)
    .map(char => [char.code, char])
);

// Find characters with combining marks
const combiningChars = getCharacters(CharacterSetType.Combining_Diacritical_Marks)
  .filter(char => char.comb !== undefined);
```

## 📚 Available Character Sets

The library provides access to all Unicode 17.0.0 blocks, including:

- **Scripts**: Latin, Greek, Cyrillic, Arabic, Chinese, Japanese, Korean, and many more
- **Symbols**: Mathematical operators, arrows, currency symbols, geometric shapes
- **Emoji**: Emoticons, transport symbols, miscellaneous pictographs
- **Specialized**: Musical notation, chess symbols, playing cards, alchemical symbols
- **Historical**: Ancient scripts, cuneiform, hieroglyphs, and more

For a complete list, see the [`CharacterSetType` enum](https://jsr.io/@j3lte/unidata/doc/~/CharacterSetType) in the source code. You can also see the [Unicode Character Database](https://www.unicode.org/charts/) for more information.

## 🔧 API Reference

### Functions

- `getCharacters(type)`: Retrieve characters from specified Unicode blocks ([doc](https://jsr.io/@j3lte/unidata/doc/~/getCharacters))
- `getCharacterSet(type)`: Get complete character set with metadata ([doc](https://jsr.io/@j3lte/unidata/doc/~/getCharacterSet))
- `getCharacterBaseSet(type)`: Get basic block information ([doc](https://jsr.io/@j3lte/unidata/doc/~/getCharacterBaseSet))
- `UNICODE_VERSION`: Current Unicode version (17.0.0) per 2025-09-09

### Types

- `Character`: Individual character data with properties ([doc](https://jsr.io/@j3lte/unidata/doc/~/Character))
- `CharacterSet`: Complete character set with metadata ([doc](https://jsr.io/@j3lte/unidata/doc/~/CharacterSet))
- `CharacterSetType`: Enum of all available Unicode blocks ([doc](https://jsr.io/@j3lte/unidata/doc/~/CharacterSetType))

## 📊 Character Sets

<!-- START CharacterSets -->
| Name | Start Code | End Code | Total Characters |
|------|------------|----------|------------------|
| Adlam | U+1E900 | U+1E95F | 88 |
| Aegean Numbers | U+10100 | U+1013F | 57 |
| Ahom | U+11700 | U+1174F | 65 |
| Alchemical Symbols | U+1F700 | U+1F77F | 128 |
| Alphabetic Presentation Forms | U+FB00 | U+FB4F | 58 |
| Anatolian Hieroglyphs | U+14400 | U+1467F | 583 |
| Ancient Greek Musical Notation | U+1D200 | U+1D24F | 70 |
| Ancient Greek Numbers | U+10140 | U+1018F | 79 |
| Ancient Symbols | U+10190 | U+101CF | 14 |
| Arabic | U+0600 | U+06FF | 256 |
| Arabic Extended-A | U+08A0 | U+08FF | 96 |
| Arabic Extended-B | U+0870 | U+089F | 43 |
| Arabic Extended-C | U+10EC0 | U+10EFF | 21 |
| Arabic Mathematical Alphabetic Symbols | U+1EE00 | U+1EEFF | 143 |
| Arabic Presentation Forms-A | U+FB50 | U+FDFF | 656 |
| Arabic Presentation Forms-B | U+FE70 | U+FEFF | 141 |
| Arabic Supplement | U+0750 | U+077F | 48 |
| Armenian | U+0530 | U+058F | 91 |
| Arrows | U+2190 | U+21FF | 112 |
| Avestan | U+10B00 | U+10B3F | 61 |
| Balinese | U+1B00 | U+1B7F | 127 |
| Bamum | U+A6A0 | U+A6FF | 88 |
| Bamum Supplement | U+16800 | U+16A3F | 569 |
| Basic Latin | U+0000 | U+007F | 128 |
| Bassa Vah | U+16AD0 | U+16AFF | 36 |
| Batak | U+1BC0 | U+1BFF | 56 |
| Bengali | U+0980 | U+09FF | 96 |
| Beria Erfe | U+16EA0 | U+16EDF | 50 |
| Bhaiksuki | U+11C00 | U+11C6F | 97 |
| Block Elements | U+2580 | U+259F | 32 |
| Bopomofo | U+3100 | U+312F | 43 |
| Bopomofo Extended | U+31A0 | U+31BF | 32 |
| Box Drawing | U+2500 | U+257F | 128 |
| Brahmi | U+11000 | U+1107F | 115 |
| Braille Patterns | U+2800 | U+28FF | 256 |
| Buginese | U+1A00 | U+1A1F | 30 |
| Buhid | U+1740 | U+175F | 20 |
| Byzantine Musical Symbols | U+1D000 | U+1D0FF | 246 |
| Carian | U+102A0 | U+102DF | 49 |
| Caucasian Albanian | U+10530 | U+1056F | 53 |
| Chakma | U+11100 | U+1114F | 71 |
| Cham | U+AA00 | U+AA5F | 83 |
| Cherokee | U+13A0 | U+13FF | 92 |
| Cherokee Supplement | U+AB70 | U+ABBF | 80 |
| Chess Symbols | U+1FA00 | U+1FA6F | 102 |
| Chorasmian | U+10FB0 | U+10FDF | 28 |
| CJK Compatibility | U+3300 | U+33FF | 256 |
| CJK Compatibility Forms | U+FE30 | U+FE4F | 32 |
| CJK Compatibility Ideographs | U+F900 | U+FAFF | 472 |
| CJK Compatibility Ideographs Supplement | U+2F800 | U+2FA1F | 542 |
| CJK Radicals Supplement | U+2E80 | U+2EFF | 115 |
| CJK Strokes | U+31C0 | U+31EF | 39 |
| CJK Symbols and Punctuation | U+3000 | U+303F | 64 |
| CJK Unified Ideographs | U+4E00 | U+9FFF | 2 |
| CJK Unified Ideographs Extension A | U+3400 | U+4DBF | 2 |
| CJK Unified Ideographs Extension B | U+20000 | U+2A6DF | 2 |
| CJK Unified Ideographs Extension C | U+2A700 | U+2B73F | 2 |
| CJK Unified Ideographs Extension D | U+2B740 | U+2B81F | 2 |
| CJK Unified Ideographs Extension E | U+2B820 | U+2CEAF | 2 |
| CJK Unified Ideographs Extension F | U+2CEB0 | U+2EBEF | 2 |
| CJK Unified Ideographs Extension G | U+30000 | U+3134F | 2 |
| CJK Unified Ideographs Extension H | U+31350 | U+323AF | 2 |
| CJK Unified Ideographs Extension I | U+2EBF0 | U+2EE5F | 2 |
| CJK Unified Ideographs Extension J | U+323B0 | U+3347F | 2 |
| Combining Diacritical Marks | U+0300 | U+036F | 112 |
| Combining Diacritical Marks Extended | U+1AB0 | U+1AFF | 58 |
| Combining Diacritical Marks for Symbols | U+20D0 | U+20FF | 33 |
| Combining Diacritical Marks Supplement | U+1DC0 | U+1DFF | 64 |
| Combining Half Marks | U+FE20 | U+FE2F | 16 |
| Common Indic Number Forms | U+A830 | U+A83F | 10 |
| Control Pictures | U+2400 | U+243F | 42 |
| Coptic | U+2C80 | U+2CFF | 123 |
| Coptic Epact Numbers | U+102E0 | U+102FF | 28 |
| Counting Rod Numerals | U+1D360 | U+1D37F | 25 |
| Cuneiform | U+12000 | U+123FF | 922 |
| Cuneiform Numbers and Punctuation | U+12400 | U+1247F | 116 |
| Currency Symbols | U+20A0 | U+20CF | 34 |
| Cypriot Syllabary | U+10800 | U+1083F | 55 |
| Cypro-Minoan | U+12F90 | U+12FFF | 99 |
| Cyrillic | U+0400 | U+04FF | 256 |
| Cyrillic Extended-A | U+2DE0 | U+2DFF | 32 |
| Cyrillic Extended-B | U+A640 | U+A69F | 96 |
| Cyrillic Extended-C | U+1C80 | U+1C8F | 11 |
| Cyrillic Extended-D | U+1E030 | U+1E08F | 63 |
| Cyrillic Supplement | U+0500 | U+052F | 48 |
| Deseret | U+10400 | U+1044F | 80 |
| Devanagari | U+0900 | U+097F | 128 |
| Devanagari Extended | U+A8E0 | U+A8FF | 32 |
| Devanagari Extended-A | U+11B00 | U+11B5F | 10 |
| Dingbats | U+2700 | U+27BF | 192 |
| Dives Akuru | U+11900 | U+1195F | 72 |
| Dogra | U+11800 | U+1184F | 60 |
| Domino Tiles | U+1F030 | U+1F09F | 100 |
| Duployan | U+1BC00 | U+1BC9F | 143 |
| Early Dynastic Cuneiform | U+12480 | U+1254F | 196 |
| Egyptian Hieroglyph Format Controls | U+13430 | U+1345F | 38 |
| Egyptian Hieroglyphs | U+13000 | U+1342F | 1072 |
| Egyptian Hieroglyphs Extended-A | U+13460 | U+143FF | 3995 |
| Elbasan | U+10500 | U+1052F | 40 |
| Elymaic | U+10FE0 | U+10FFF | 23 |
| Emoticons | U+1F600 | U+1F64F | 80 |
| Enclosed Alphanumeric Supplement | U+1F100 | U+1F1FF | 200 |
| Enclosed Alphanumerics | U+2460 | U+24FF | 160 |
| Enclosed CJK Letters and Months | U+3200 | U+32FF | 255 |
| Enclosed Ideographic Supplement | U+1F200 | U+1F2FF | 64 |
| Ethiopic | U+1200 | U+137F | 358 |
| Ethiopic Extended | U+2D80 | U+2DDF | 79 |
| Ethiopic Extended-A | U+AB00 | U+AB2F | 32 |
| Ethiopic Extended-B | U+1E7E0 | U+1E7FF | 28 |
| Ethiopic Supplement | U+1380 | U+139F | 26 |
| Garay | U+10D40 | U+10D8F | 69 |
| General Punctuation | U+2000 | U+206F | 111 |
| Geometric Shapes | U+25A0 | U+25FF | 96 |
| Geometric Shapes Extended | U+1F780 | U+1F7FF | 103 |
| Georgian | U+10A0 | U+10FF | 88 |
| Georgian Extended | U+1C90 | U+1CBF | 46 |
| Georgian Supplement | U+2D00 | U+2D2F | 40 |
| Glagolitic | U+2C00 | U+2C5F | 96 |
| Glagolitic Supplement | U+1E000 | U+1E02F | 38 |
| Gothic | U+10330 | U+1034F | 27 |
| Grantha | U+11300 | U+1137F | 86 |
| Greek and Coptic | U+0370 | U+03FF | 135 |
| Greek Extended | U+1F00 | U+1FFF | 233 |
| Gujarati | U+0A80 | U+0AFF | 91 |
| Gunjala Gondi | U+11D60 | U+11DAF | 63 |
| Gurmukhi | U+0A00 | U+0A7F | 80 |
| Gurung Khema | U+16100 | U+1613F | 58 |
| Halfwidth and Fullwidth Forms | U+FF00 | U+FFEF | 225 |
| Hangul Compatibility Jamo | U+3130 | U+318F | 94 |
| Hangul Jamo | U+1100 | U+11FF | 256 |
| Hangul Jamo Extended-A | U+A960 | U+A97F | 29 |
| Hangul Jamo Extended-B | U+D7B0 | U+D7FF | 72 |
| Hangul Syllables | U+AC00 | U+D7AF | 2 |
| Hanifi Rohingya | U+10D00 | U+10D3F | 50 |
| Hanunoo | U+1720 | U+173F | 23 |
| Hatran | U+108E0 | U+108FF | 26 |
| Hebrew | U+0590 | U+05FF | 88 |
| High Private Use Surrogates | U+DB80 | U+DBFF | 2 |
| High Surrogates | U+D800 | U+DB7F | 2 |
| Hiragana | U+3040 | U+309F | 93 |
| Ideographic Description Characters | U+2FF0 | U+2FFF | 16 |
| Ideographic Symbols and Punctuation | U+16FE0 | U+16FFF | 12 |
| Imperial Aramaic | U+10840 | U+1085F | 31 |
| Indic Siyaq Numbers | U+1EC70 | U+1ECBF | 68 |
| Inscriptional Pahlavi | U+10B60 | U+10B7F | 27 |
| Inscriptional Parthian | U+10B40 | U+10B5F | 30 |
| IPA Extensions | U+0250 | U+02AF | 96 |
| Javanese | U+A980 | U+A9DF | 91 |
| Kaithi | U+11080 | U+110CF | 68 |
| Kaktovik Numerals | U+1D2C0 | U+1D2DF | 20 |
| Kana Extended-A | U+1B100 | U+1B12F | 35 |
| Kana Extended-B | U+1AFF0 | U+1AFFF | 13 |
| Kana Supplement | U+1B000 | U+1B0FF | 256 |
| Kanbun | U+3190 | U+319F | 16 |
| Kangxi Radicals | U+2F00 | U+2FDF | 214 |
| Kannada | U+0C80 | U+0CFF | 92 |
| Katakana | U+30A0 | U+30FF | 96 |
| Katakana Phonetic Extensions | U+31F0 | U+31FF | 16 |
| Kawi | U+11F00 | U+11F5F | 87 |
| Kayah Li | U+A900 | U+A92F | 48 |
| Kharoshthi | U+10A00 | U+10A5F | 68 |
| Khitan Small Script | U+18B00 | U+18CFF | 471 |
| Khmer | U+1780 | U+17FF | 114 |
| Khmer Symbols | U+19E0 | U+19FF | 32 |
| Khojki | U+11200 | U+1124F | 65 |
| Khudawadi | U+112B0 | U+112FF | 69 |
| Kirat Rai | U+16D40 | U+16D7F | 58 |
| Lao | U+0E80 | U+0EFF | 83 |
| Latin Extended Additional | U+1E00 | U+1EFF | 256 |
| Latin Extended-A | U+0100 | U+017F | 128 |
| Latin Extended-B | U+0180 | U+024F | 208 |
| Latin Extended-C | U+2C60 | U+2C7F | 32 |
| Latin Extended-D | U+A720 | U+A7FF | 204 |
| Latin Extended-E | U+AB30 | U+AB6F | 60 |
| Latin Extended-F | U+10780 | U+107BF | 57 |
| Latin Extended-G | U+1DF00 | U+1DFFF | 37 |
| Latin-1 Supplement | U+0080 | U+00FF | 128 |
| Lepcha | U+1C00 | U+1C4F | 74 |
| Letterlike Symbols | U+2100 | U+214F | 80 |
| Limbu | U+1900 | U+194F | 68 |
| Linear A | U+10600 | U+1077F | 341 |
| Linear B Ideograms | U+10080 | U+100FF | 123 |
| Linear B Syllabary | U+10000 | U+1007F | 88 |
| Lisu | U+A4D0 | U+A4FF | 48 |
| Lisu Supplement | U+11FB0 | U+11FBF | 1 |
| Low Surrogates | U+DC00 | U+DFFF | 2 |
| Lycian | U+10280 | U+1029F | 29 |
| Lydian | U+10920 | U+1093F | 27 |
| Mahajani | U+11150 | U+1117F | 39 |
| Mahjong Tiles | U+1F000 | U+1F02F | 44 |
| Makasar | U+11EE0 | U+11EFF | 25 |
| Malayalam | U+0D00 | U+0D7F | 118 |
| Mandaic | U+0840 | U+085F | 29 |
| Manichaean | U+10AC0 | U+10AFF | 51 |
| Marchen | U+11C70 | U+11CBF | 68 |
| Masaram Gondi | U+11D00 | U+11D5F | 75 |
| Mathematical Alphanumeric Symbols | U+1D400 | U+1D7FF | 996 |
| Mathematical Operators | U+2200 | U+22FF | 256 |
| Mayan Numerals | U+1D2E0 | U+1D2FF | 20 |
| Medefaidrin | U+16E40 | U+16E9F | 91 |
| Meetei Mayek | U+ABC0 | U+ABFF | 56 |
| Meetei Mayek Extensions | U+AAE0 | U+AAFF | 23 |
| Mende Kikakui | U+1E800 | U+1E8DF | 213 |
| Meroitic Cursive | U+109A0 | U+109FF | 90 |
| Meroitic Hieroglyphs | U+10980 | U+1099F | 32 |
| Miao | U+16F00 | U+16F9F | 149 |
| Miscellaneous Mathematical Symbols-A | U+27C0 | U+27EF | 48 |
| Miscellaneous Mathematical Symbols-B | U+2980 | U+29FF | 128 |
| Miscellaneous Symbols | U+2600 | U+26FF | 256 |
| Miscellaneous Symbols and Arrows | U+2B00 | U+2BFF | 254 |
| Miscellaneous Symbols and Pictographs | U+1F300 | U+1F5FF | 768 |
| Miscellaneous Symbols Supplement | U+1CEC0 | U+1CEFF | 34 |
| Miscellaneous Technical | U+2300 | U+23FF | 256 |
| Modi | U+11600 | U+1165F | 79 |
| Modifier Tone Letters | U+A700 | U+A71F | 32 |
| Mongolian | U+1800 | U+18AF | 158 |
| Mongolian Supplement | U+11660 | U+1167F | 13 |
| Mro | U+16A40 | U+16A6F | 43 |
| Multani | U+11280 | U+112AF | 38 |
| Musical Symbols | U+1D100 | U+1D1FF | 233 |
| Myanmar | U+1000 | U+109F | 160 |
| Myanmar Extended-A | U+AA60 | U+AA7F | 32 |
| Myanmar Extended-B | U+A9E0 | U+A9FF | 31 |
| Myanmar Extended-C | U+116D0 | U+116FF | 20 |
| Nabataean | U+10880 | U+108AF | 40 |
| Nag Mundari | U+1E4D0 | U+1E4FF | 42 |
| Nandinagari | U+119A0 | U+119FF | 65 |
| New Tai Lue | U+1980 | U+19DF | 83 |
| Newa | U+11400 | U+1147F | 97 |
| NKo | U+07C0 | U+07FF | 62 |
| Number Forms | U+2150 | U+218F | 60 |
| Nushu | U+1B170 | U+1B2FF | 396 |
| Nyiakeng Puachue Hmong | U+1E100 | U+1E14F | 71 |
| Ogham | U+1680 | U+169F | 29 |
| Ol Chiki | U+1C50 | U+1C7F | 48 |
| Ol Onal | U+1E5D0 | U+1E5FF | 44 |
| Old Hungarian | U+10C80 | U+10CFF | 108 |
| Old Italic | U+10300 | U+1032F | 39 |
| Old North Arabian | U+10A80 | U+10A9F | 32 |
| Old Permic | U+10350 | U+1037F | 43 |
| Old Persian | U+103A0 | U+103DF | 50 |
| Old Sogdian | U+10F00 | U+10F2F | 40 |
| Old South Arabian | U+10A60 | U+10A7F | 32 |
| Old Turkic | U+10C00 | U+10C4F | 73 |
| Old Uyghur | U+10F70 | U+10FAF | 26 |
| Optical Character Recognition | U+2440 | U+245F | 11 |
| Oriya | U+0B00 | U+0B7F | 91 |
| Ornamental Dingbats | U+1F650 | U+1F67F | 48 |
| Osage | U+104B0 | U+104FF | 72 |
| Osmanya | U+10480 | U+104AF | 40 |
| Ottoman Siyaq Numbers | U+1ED00 | U+1ED4F | 61 |
| Pahawh Hmong | U+16B00 | U+16B8F | 127 |
| Palmyrene | U+10860 | U+1087F | 32 |
| Pau Cin Hau | U+11AC0 | U+11AFF | 57 |
| Phags-pa | U+A840 | U+A87F | 56 |
| Phaistos Disc | U+101D0 | U+101FF | 46 |
| Phoenician | U+10900 | U+1091F | 29 |
| Phonetic Extensions | U+1D00 | U+1D7F | 128 |
| Phonetic Extensions Supplement | U+1D80 | U+1DBF | 64 |
| Playing Cards | U+1F0A0 | U+1F0FF | 82 |
| Private Use Area | U+E000 | U+F8FF | 2 |
| Psalter Pahlavi | U+10B80 | U+10BAF | 29 |
| Rejang | U+A930 | U+A95F | 37 |
| Rumi Numeral Symbols | U+10E60 | U+10E7F | 31 |
| Runic | U+16A0 | U+16FF | 89 |
| Samaritan | U+0800 | U+083F | 61 |
| Saurashtra | U+A880 | U+A8DF | 82 |
| Sharada | U+11180 | U+111DF | 96 |
| Sharada Supplement | U+11B60 | U+11B7F | 8 |
| Shavian | U+10450 | U+1047F | 48 |
| Shorthand Format Controls | U+1BCA0 | U+1BCAF | 4 |
| Siddham | U+11580 | U+115FF | 92 |
| Sidetic | U+10940 | U+1095F | 26 |
| Sinhala | U+0D80 | U+0DFF | 91 |
| Sinhala Archaic Numbers | U+111E0 | U+111FF | 20 |
| Small Form Variants | U+FE50 | U+FE6F | 26 |
| Small Kana Extension | U+1B130 | U+1B16F | 9 |
| Sogdian | U+10F30 | U+10F6F | 42 |
| Sora Sompeng | U+110D0 | U+110FF | 35 |
| Soyombo | U+11A50 | U+11AAF | 83 |
| Spacing Modifier Letters | U+02B0 | U+02FF | 80 |
| Specials | U+FFF0 | U+FFFF | 5 |
| Sundanese | U+1B80 | U+1BBF | 64 |
| Sundanese Supplement | U+1CC0 | U+1CCF | 8 |
| Sunuwar | U+11BC0 | U+11BFF | 44 |
| Superscripts and Subscripts | U+2070 | U+209F | 42 |
| Supplemental Arrows-A | U+27F0 | U+27FF | 16 |
| Supplemental Arrows-B | U+2900 | U+297F | 128 |
| Supplemental Arrows-C | U+1F800 | U+1F8FF | 171 |
| Supplemental Mathematical Operators | U+2A00 | U+2AFF | 256 |
| Supplemental Punctuation | U+2E00 | U+2E7F | 94 |
| Supplemental Symbols and Pictographs | U+1F900 | U+1F9FF | 256 |
| Supplementary Private Use Area-A | U+F0000 | U+FFFFF | 2 |
| Supplementary Private Use Area-B | U+100000 | U+10FFFF | 2 |
| Sutton SignWriting | U+1D800 | U+1DAAF | 672 |
| Syloti Nagri | U+A800 | U+A82F | 45 |
| Symbols and Pictographs Extended-A | U+1FA70 | U+1FAFF | 120 |
| Symbols for Legacy Computing | U+1FB00 | U+1FBFF | 250 |
| Symbols for Legacy Computing Supplement | U+1CC00 | U+1CEBF | 695 |
| Syriac | U+0700 | U+074F | 77 |
| Syriac Supplement | U+0860 | U+086F | 11 |
| Tagalog | U+1700 | U+171F | 23 |
| Tagbanwa | U+1760 | U+177F | 18 |
| Tags | U+E0000 | U+E007F | 97 |
| Tai Le | U+1950 | U+197F | 35 |
| Tai Tham | U+1A20 | U+1AAF | 127 |
| Tai Viet | U+AA80 | U+AADF | 72 |
| Tai Xuan Jing Symbols | U+1D300 | U+1D35F | 87 |
| Tai Yo | U+1E6C0 | U+1E6FF | 55 |
| Takri | U+11680 | U+116CF | 68 |
| Tamil | U+0B80 | U+0BFF | 72 |
| Tamil Supplement | U+11FC0 | U+11FFF | 51 |
| Tangsa | U+16A70 | U+16ACF | 89 |
| Tangut | U+17000 | U+187FF | 2 |
| Tangut Components | U+18800 | U+18AFF | 768 |
| Tangut Components Supplement | U+18D80 | U+18DFF | 115 |
| Tangut Supplement | U+18D00 | U+18D7F | 2 |
| Telugu | U+0C00 | U+0C7F | 101 |
| Thaana | U+0780 | U+07BF | 50 |
| Thai | U+0E00 | U+0E7F | 87 |
| Tibetan | U+0F00 | U+0FFF | 211 |
| Tifinagh | U+2D30 | U+2D7F | 59 |
| Tirhuta | U+11480 | U+114DF | 82 |
| Todhri | U+105C0 | U+105FF | 52 |
| Tolong Siki | U+11DB0 | U+11DEF | 54 |
| Toto | U+1E290 | U+1E2BF | 31 |
| Transport and Map Symbols | U+1F680 | U+1F6FF | 119 |
| Tulu-Tigalari | U+11380 | U+113FF | 80 |
| Ugaritic | U+10380 | U+1039F | 31 |
| Unified Canadian Aboriginal Syllabics | U+1400 | U+167F | 640 |
| Unified Canadian Aboriginal Syllabics Extended | U+18B0 | U+18FF | 70 |
| Unified Canadian Aboriginal Syllabics Extended-A | U+11AB0 | U+11ABF | 16 |
| Vai | U+A500 | U+A63F | 300 |
| Variation Selectors | U+FE00 | U+FE0F | 16 |
| Variation Selectors Supplement | U+E0100 | U+E01EF | 240 |
| Vedic Extensions | U+1CD0 | U+1CFF | 43 |
| Vertical Forms | U+FE10 | U+FE1F | 10 |
| Vithkuqi | U+10570 | U+105BF | 70 |
| Wancho | U+1E2C0 | U+1E2FF | 59 |
| Warang Citi | U+118A0 | U+118FF | 84 |
| Yezidi | U+10E80 | U+10EBF | 47 |
| Yi Radicals | U+A490 | U+A4CF | 55 |
| Yi Syllables | U+A000 | U+A48F | 1165 |
| Yijing Hexagram Symbols | U+4DC0 | U+4DFF | 64 |
| Zanabazar Square | U+11A00 | U+11A4F | 72 |
| Znamenny Musical Notation | U+1CF00 | U+1CFCF | 185 |
<!-- END CharacterSets -->

## 🎯 Use Cases

- **Internationalization**: Support for multiple languages and scripts
- **Text Processing**: Character analysis and manipulation
- **Input Validation**: Unicode-aware form validation
- **Font Development**: Character set analysis for font creation
- **Educational Tools**: Unicode learning and exploration
- **Data Analysis**: Character frequency and distribution analysis

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgements

This library is inspired by the excellent Node.js library [unidata](https://github.com/chbrown/unidata) by Christopher Brown. The big difference is that his library is based Unicode version __13.0.0__, while this library is based on Unicode version __17.0.0__.
