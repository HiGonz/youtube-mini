//! Validación de URLs: qué puede cargar el webview de YouTube y cómo se interpreta la entrada del usuario.
use url::Url;

fn host_is(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Hosts de YouTube propiamente dicho (los únicos con acceso a los 2 comandos IPC).
pub fn is_youtube_host(host: &str) -> bool {
    host_is(host, "youtube.com") || host == "youtu.be"
}

/// Navegación permitida dentro del webview: YouTube + el flujo de inicio de sesión de Google.
/// Todo lo demás se abre en el navegador del sistema.
pub fn is_allowed_nav(url: &Url) -> bool {
    if url.as_str() == "about:blank" { return true; }
    if url.scheme() != "https" { return false; }
    let Some(host) = url.host_str() else { return false };
    is_youtube_host(host)
        || host_is(host, "google.com")
        || host_is(host, "gstatic.com")
        || host_is(host, "googleusercontent.com")
        || host_is(host, "ytimg.com")
}

/// Artwork aceptado desde la página (para mostrarlo en la UI local, cuya CSP lo restringe).
pub fn is_allowed_artwork(url: &str) -> bool {
    Url::parse(url).ok().is_some_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some_and(|h| {
                host_is(h, "ytimg.com") || host_is(h, "ggpht.com") || host_is(h, "googleusercontent.com")
            })
    })
}

pub const HOME: &str = "https://www.youtube.com/";
pub const MUSIC_HOME: &str = "https://music.youtube.com/";

pub fn search_url(query: &str, music: bool) -> Url {
    let (base, key) = if music { ("https://music.youtube.com/search", "q") } else { ("https://www.youtube.com/results", "search_query") };
    let mut u = Url::parse(base).expect("static url");
    u.query_pairs_mut().append_pair(key, query.trim());
    u
}

/// Interpreta el texto del buscador: URL de YouTube/youtu.be (se normaliza) o búsqueda.
pub fn resolve_input(input: &str, music: bool) -> Url {
    let text = input.trim();
    let candidate = if text.contains("://") {
        Url::parse(text).ok()
    } else if text.contains('.') && !text.contains(' ') {
        Url::parse(&format!("https://{text}")).ok()
    } else {
        None
    };
    if let Some(mut u) = candidate {
        if let Some(host) = u.host_str().map(str::to_owned) {
            if host == "youtu.be" {
                let id = u.path().trim_start_matches('/').to_string();
                if !id.is_empty() {
                    let mut out = Url::parse("https://www.youtube.com/watch").unwrap();
                    out.query_pairs_mut().append_pair("v", &id);
                    if let Some((_, t)) = u.query_pairs().find(|(k, _)| k == "t") {
                        out.query_pairs_mut().append_pair("t", &t);
                    }
                    return out;
                }
            } else if host_is(&host, "youtube.com") {
                let _ = u.set_scheme("https");
                if host == "youtube.com" { let _ = u.set_host(Some("www.youtube.com")); }
                u.set_fragment(None);
                return u;
            }
        }
    }
    search_url(text, music)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn u(s: &str) -> Url { Url::parse(s).unwrap() }

    #[test]
    fn nav_allowlist() {
        assert!(is_allowed_nav(&u("https://www.youtube.com/watch?v=x")));
        assert!(is_allowed_nav(&u("https://music.youtube.com/")));
        assert!(is_allowed_nav(&u("https://accounts.google.com/signin")));
        assert!(!is_allowed_nav(&u("http://www.youtube.com/")));
        assert!(!is_allowed_nav(&u("https://evil-youtube.com/")));
        assert!(!is_allowed_nav(&u("https://youtube.com.evil.io/")));
        assert!(!is_allowed_nav(&u("file:///C:/Windows/win.ini")));
        assert!(!is_allowed_nav(&u("javascript:alert(1)")));
    }

    #[test]
    fn resolves_inputs() {
        assert_eq!(resolve_input("https://youtu.be/abc123?t=42", false).as_str(), "https://www.youtube.com/watch?v=abc123&t=42");
        assert_eq!(resolve_input("youtube.com/watch?v=zz", false).as_str(), "https://www.youtube.com/watch?v=zz");
        assert_eq!(resolve_input("http://www.youtube.com/playlist?list=PL1#x", false).as_str(), "https://www.youtube.com/playlist?list=PL1");
        assert_eq!(resolve_input("lofi hip hop", false).as_str(), "https://www.youtube.com/results?search_query=lofi+hip+hop");
        assert_eq!(resolve_input("daft punk", true).as_str(), "https://music.youtube.com/search?q=daft+punk");
        // Una URL ajena no se navega: se trata como texto de búsqueda.
        assert!(resolve_input("https://evil.example/x", false).as_str().starts_with("https://www.youtube.com/results"));
    }

    #[test]
    fn artwork_hosts() {
        assert!(is_allowed_artwork("https://i.ytimg.com/vi/x/hq.jpg"));
        assert!(is_allowed_artwork("https://lh3.googleusercontent.com/a"));
        assert!(!is_allowed_artwork("https://evil.com/a.png"));
        assert!(!is_allowed_artwork("http://i.ytimg.com/a.png"));
    }
}
