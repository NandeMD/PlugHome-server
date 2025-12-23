use tokio::sync::OnceCell;

pub static TIME_NOW: OnceCell<String> = OnceCell::const_new();
