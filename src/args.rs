pub fn convert_args_to_config() {
    let args: Vec<_> = std::env::args().collect();
    for argument in std::env::args() {
        println!("Argument {}", argument);
    }
    if args.len() > 2 && args[1] == "-run" {
        print!("About to run {} are you happy now?!:) \n", args[2]);
    }
}
