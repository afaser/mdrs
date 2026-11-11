use iced::Font;

pub const BYTES: &[u8] = include_bytes!("../../assets/JetBrainsMonoNerdFont-Regular.ttf");

const FAMILY: &str = "JetBrainsMono Nerd Font";

const FONTCONFIG: &str = "<?xml version=\"1.0\"?><!DOCTYPE fontconfig SYSTEM \"fonts.dtd\"><fontconfig><dir>/nonexistent/mdrs-fonts</dir></fontconfig>";

pub fn regular() -> Font {
    Font::with_name(FAMILY)
}

pub fn isolate_system_fonts() {
    if std::env::var_os("FONTCONFIG_FILE").is_some() {
        return;
    }

    let configuration = std::env::temp_dir().join("mdrs-fonts.conf");

    if std::fs::write(&configuration, FONTCONFIG).is_err() {
        return;
    }

    unsafe {
        std::env::set_var("FONTCONFIG_FILE", configuration);
    }
}
