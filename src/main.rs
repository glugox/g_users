use g_users;

fn main() {
    g_users::load_env(None);
    g_users::rocket().launch();
}