use indicatif::ProgressStyle;

pub fn getProgressStyle() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .template("{spinner:.blue} {msg}")
        // .unwrap()
        // For more spinners check out the cli-spinners project:
        // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
        .tick_strings(&[
            "( ●    )",
            "(  ●   )",
            "(   ●  )",
            "(    ● )",
            "(     ●)",
            "(    ● )",
            "(   ●  )",
            "(  ●   )",
            "( ●    )",
            "(●     )",
        ])
}
