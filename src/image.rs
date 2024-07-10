use std::{fs, time::Instant};

use image_builder::{colors, FilterType, Image, Picture, Rect, Text};

const BACKGROUND: [u8; 4] = [0x15, 0x18, 0x1f, 0xff];
const CONTENT: [u8; 4] = [0xf0, 0xf2, 0xf5, 0xff];
const GRAY: [u8; 4] = [0x4c, 0x50, 0x59, 0xff];

pub fn entry_image(name: &str, description: &str) {
    let width = 1250;
    let height = 600;

    let mut image = Image::new(width, height, BACKGROUND);
    
    let entry_font = fs::read("assets/fonts/SourceSerif4_36pt-MediumItalic.ttf").unwrap();
    let description_font = fs::read("assets/fonts/Inter-Light.ttf").unwrap();

    image.add_custom_font("Entry", entry_font);
    image.add_custom_font("Description", description_font);

    image.add_picture(
        Picture::new("assets/logo-dark.png")
            .resize(160, 160, FilterType::Triangle)
            .position(1050, 400),
    );

    image.add_text(
        Text::new(name)
            .size(100)
            .font("Entry")
            .position(80, 80)
            .color(CONTENT),
    );
    
    let words = description.split(" ");
    let mut line = String::new();
    let mut line_no = 0;

    let mut add_line = |line: &str| {
        image.add_text(
            Text::new(&line)
                .size(55)
                .font("Description")
                .position(80, 220 + 60 * line_no)
                .color(CONTENT),
        );
        line_no += 1;
    };

    for word in words {
        if line.len() + word.len() > 45 {
            add_line(&line);
            line.clear();
        }

        line.push_str(word);
        line.push(' ');
    }

    add_line(&line);

    image.save("content/preview.png");
}

pub fn section_image(
    name: &str,
    description: &str,
    section_index: &str,
    date: &str
) {
    let width = 1250;
    let height = 640;

    let mut image = Image::new(width, height, BACKGROUND);
    
    let entry_font = fs::read("assets/fonts/SourceSerif4_36pt-MediumItalic.ttf").unwrap();
    let description_font = fs::read("assets/fonts/Inter-Light.ttf").unwrap();

    image.add_custom_font("Entry", entry_font);
    image.add_custom_font("Description", description_font);

    image.add_picture(
        Picture::new("assets/logo-dark.png")
            .resize(160, 160, FilterType::Triangle)
            .position(1050, 420),
    );

    image.add_text(
        Text::new(name)
            .size(100)
            .font("Entry")
            .position(80, 80)
            .color(CONTENT),
    );
    
    let words = description.split(" ");
    let mut line = String::new();
    let mut line_no = 0;

    let mut add_line = |line: &str| {
        image.add_text(
            Text::new(&line)
                .size(55)
                .font("Description")
                .position(80, 220 + 60 * line_no)
                .color(CONTENT),
        );
        line_no += 1;
    };

    for word in words {
        if line.len() + word.len() > 45 {
            add_line(&line);
            line.clear();
        }

        line.push_str(word);
        line.push(' ');
    }

    add_line(&line);

    image.add_text(
        Text::new(section_index)
            .size(55)
            .font("Description")
            .position(80, 520)
            .color(GRAY),
    );

    image.add_text(
        Text::new(date)
            .size(55)
            .font("Description")
            .position(380, 520)
            .color(GRAY),
    );

    image.save("content/preview.png");
}
