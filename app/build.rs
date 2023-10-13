extern crate windres;

fn main() {
    #[cfg(target_os = "windows")]
    {
        use windres::Build;
        Build::new().compile("./res/resources.rc").unwrap();
    }
}
