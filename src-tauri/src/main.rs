fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Some(command) = args.first().cloned() {
        if browserroute_lib::cli(command.clone(), args.iter().skip(1).cloned().collect()) {
            return;
        }
        browserroute_lib::run(Some(command));
        return;
    }
    browserroute_lib::run(None);
}
