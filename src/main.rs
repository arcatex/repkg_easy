use repkg::{os, re};

fn main() -> Result<(), eframe::Error> {
    repkg::run()
}

fn demo() {

    // let arg = r" extract -o C:\admin\workplace\all\demo C:\admin\workplace\repkg1";
    // os::process_repkg(arg);

    let param = re::Param {
        target: String::from("C:\\admin\\workplace\\repkg1"),
        saved: String::from("C:\\admin\\workplace\\all\\demo"),
    };

    re::extract(param);
    
}
