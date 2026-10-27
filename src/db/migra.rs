use std::path::{Path, PathBuf};

const ID_ANTIGO: &str = "io.github.cosmic_utils.cosmic-ext-applet-clipboard-manager";
const APP_ANTIGO: &str = "cosmic-ext-applet-clipboard-manager";
const SUFIXOS: [&str; 3] = ["", "-wal", "-shm"];

fn candidatos(home: &Path, arquivo_antigo: &str) -> [PathBuf; 2] {
    [
        home.join(".var/app")
            .join(ID_ANTIGO)
            .join("cache")
            .join(APP_ANTIGO)
            .join(arquivo_antigo),
        home.join(".cache").join(APP_ANTIGO).join(arquivo_antigo),
    ]
}

pub fn historico_antigo(home: &Path, arquivo_antigo: &str, destino: &Path) -> bool {
    if destino.exists() {
        return false;
    }
    let Some(origem) = candidatos(home, arquivo_antigo)
        .into_iter()
        .find(|p| p.is_file())
    else {
        return false;
    };
    for sufixo in SUFIXOS {
        let de = PathBuf::from(format!("{}{sufixo}", origem.display()));
        let para = PathBuf::from(format!("{}{sufixo}", destino.display()));
        if de.is_file()
            && let Err(err) = std::fs::copy(&de, &para)
        {
            warn!("cannot copy {} to {}: {err}", de.display(), para.display());
            for sujo in SUFIXOS {
                let _ = std::fs::remove_file(format!("{}{sujo}", destino.display()));
            }
            return false;
        }
    }
    info!("history copied from {}", origem.display());
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANTIGO: &str = "io.github.cosmic_utils.cosmic-ext-applet-clipboard-manager-db-7.sqlite";

    fn pasta(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "meowsystem-clipboard-{nome}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copia_o_historico_do_flatpak_com_o_wal() {
        let home = pasta("flatpak");
        let antigo = candidatos(&home, ANTIGO)[0].clone();
        std::fs::create_dir_all(antigo.parent().unwrap()).unwrap();
        std::fs::write(&antigo, b"banco").unwrap();
        std::fs::write(format!("{}-wal", antigo.display()), b"wal").unwrap();
        let destino = home.join("novo.sqlite");

        assert!(historico_antigo(&home, ANTIGO, &destino));

        assert_eq!(std::fs::read(&destino).unwrap(), b"banco");
        assert_eq!(std::fs::read(home.join("novo.sqlite-wal")).unwrap(), b"wal");
        assert!(antigo.exists());
    }

    #[test]
    fn nao_pisa_em_banco_que_ja_existe_nem_inventa_origem() {
        let home = pasta("existe");
        let destino = home.join("novo.sqlite");
        assert!(!historico_antigo(&home, ANTIGO, &destino));
        assert!(!destino.exists());

        let antigo = candidatos(&home, ANTIGO)[1].clone();
        std::fs::create_dir_all(antigo.parent().unwrap()).unwrap();
        std::fs::write(&antigo, b"velho").unwrap();
        std::fs::write(&destino, b"atual").unwrap();
        assert!(!historico_antigo(&home, ANTIGO, &destino));
        assert_eq!(std::fs::read(&destino).unwrap(), b"atual");
    }
}
