//! Gravar um FIRMWARE baixado (C5 do `roadmaps/41` bloco C): a linha que a
//! pagina da placa no micropython.org manda, composta com as pecas do E4.
//!
//! ```text
//! esptool    `esptool [--chip <chip>] --port <porta> --baud 460800 --before default-reset
//!            --after hard-reset write-flash <offset> <arquivo>` — a pagina diz
//!            `esptool.py --port PORTNAME --baud 460800 write_flash 0x1000 <bin>` (ESP32
//!            classico) e `write_flash 0` (C3/S3), lida em 2026-09-17; `write-flash`
//!            e' a grafia da v5 desta maquina (a v4 aceita a linha editada)
//! picotool   `picotool load -f -x <uf2>` — a pagina manda copiar o .uf2 para o drive
//!            RPI-RP2 (BOOTSEL); o picotool faz o mesmo pelo USB (E4)
//! ```
//!
//! A pagina manda apagar a flash na primeira instalacao (`esptool.py
//! erase_flash`): vai em `warnings`, com a linha pronta — nunca na linha
//! de gravar, porque apagar e' outra decisao.

use std::path::PathBuf;

use kinein_protocol::{FlashProposalResult, ProjectModel};

use super::{ESPTOOL_BAUD, FindTool, FlashError, quote, tamanho_legivel, tool};

/// Um firmware INSTALADO (o arquivo existe na pasta da IDE), como o
/// catalogo o descreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareImage {
    /// Rotulo do catalogo (`MicroPython — ESP32_GENERIC …`).
    pub label: String,
    /// `esptool` | `picotool`.
    pub engine: String,
    /// Offset do `write-flash`, quando esptool.
    pub offset: Option<String>,
    /// Chave do chip para `--chip`, quando a pagina fixa uma.
    pub chip: Option<String>,
    /// O arquivo baixado.
    pub file: PathBuf,
    /// Tamanho do arquivo, para o aviso contra a flash identificada.
    pub size_bytes: u64,
}

/// A proposta para um firmware: o motor e' o da pagina; um motor pedido
/// diferente e' recusado (a pagina nao da' outra forma).
pub(super) fn propose(
    model: &ProjectModel,
    imagem: &FirmwareImage,
    engine: Option<&str>,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    find_tool: &FindTool<'_>,
) -> Result<FlashProposalResult, FlashError> {
    if let Some(pedido) = engine.map(str::trim).filter(|e| !e.is_empty())
        && pedido != imagem.engine
    {
        return Err(FlashError::UnknownEngine {
            engine: format!(
                "{pedido} (o firmware {} grava-se por {}, como a pagina da placa manda)",
                imagem.label, imagem.engine
            ),
        });
    }
    let mut source = vec![format!(
        "motor: {} (a pagina do firmware no micropython.org)",
        imagem.engine
    )];
    let mut warnings = Vec::new();
    let command = match imagem.engine.as_str() {
        "esptool" => esptool(
            imagem,
            model,
            device,
            flash_size_bytes,
            find_tool,
            &mut source,
            &mut warnings,
        )?,
        "picotool" => picotool(imagem, find_tool, &mut source)?,
        outro => {
            return Err(FlashError::UnknownEngine {
                engine: outro.to_owned(),
            });
        }
    };
    Ok(FlashProposalResult {
        name: format!("Gravar firmware ({})", imagem.label),
        command,
        engine: imagem.engine.clone(),
        source,
        warnings,
    })
}

fn esptool(
    imagem: &FirmwareImage,
    model: &ProjectModel,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<String, FlashError> {
    let esptool = find_tool("esptool")
        .or_else(|| find_tool("esptool.py"))
        .map(|p| quote(&p.display().to_string()))
        .ok_or_else(|| FlashError::MissingTool {
            tool: "esptool".to_owned(),
            hint: "pipx install esptool (o painel de instalacao mostra o passo)".to_owned(),
        })?;
    let device = device
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or_else(|| FlashError::NoDevice {
            message: "escolha a porta no painel de Embarcados (chip \"Executar\" na porta): \
                      gravar na placa errada e' pior que um clique"
                .to_owned(),
        })?;
    let offset = imagem
        .offset
        .as_deref()
        .ok_or_else(|| FlashError::NoArtifact {
            message: format!("o catalogo nao diz o offset de {}", imagem.label),
        })?;
    let arquivo = imagem.file.display().to_string();
    let mut partes = vec![esptool];
    // O chip da pagina; se o kit diz outro, o usuario ve os dois nos avisos
    // e a linha leva o da pagina (e' o firmware que decide o chip).
    if let Some(chip) = imagem.chip.as_deref() {
        partes.push("--chip".to_owned());
        partes.push(chip.to_owned());
        if let Some(kit) = model.target.chip.as_deref().filter(|k| *k != chip) {
            warnings.push(format!(
                "o kit diz chip {kit} e este firmware e' para {chip}: confira a placa antes de gravar"
            ));
        }
    }
    partes.extend([
        "--port".to_owned(),
        quote(device),
        "--baud".to_owned(),
        ESPTOOL_BAUD.to_owned(),
        "--before".to_owned(),
        "default-reset".to_owned(),
        "--after".to_owned(),
        "hard-reset".to_owned(),
        "write-flash".to_owned(),
        offset.to_owned(),
        quote(&arquivo),
    ]);
    source.push(format!(
        "firmware: {arquivo} ({})",
        tamanho_legivel(imagem.size_bytes)
    ));
    source.push(format!(
        "offset: {offset} (a pagina da placa no micropython.org)"
    ));
    source.push(format!(
        "porta: {device} (escolhida no painel de Embarcados)"
    ));
    warnings.push(format!(
        "primeira instalacao do MicroPython nesta placa? a pagina manda apagar a flash antes: \
         {} --port {} erase-flash",
        partes[0],
        quote(device)
    ));
    if let Some(placa) = flash_size_bytes
        && imagem.size_bytes > placa
    {
        warnings.push(format!(
            "o firmware tem {} e a placa identificada tem {} de flash",
            tamanho_legivel(imagem.size_bytes),
            tamanho_legivel(placa)
        ));
    }
    Ok(partes.join(" "))
}

fn picotool(
    imagem: &FirmwareImage,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<String, FlashError> {
    let picotool = tool(
        find_tool,
        "picotool",
        "pacote `picotool` da distro ou o build oficial (github.com/raspberrypi/picotool)",
    )?;
    let arquivo = imagem.file.display().to_string();
    source.push(format!(
        "firmware: {arquivo} ({})",
        tamanho_legivel(imagem.size_bytes)
    ));
    source.push(
        "modo: -f forca o BOOTSEL pelo USB; -x executa depois — o mesmo que copiar o .uf2 \
         para o drive RPI-RP2, como a pagina manda"
            .to_owned(),
    );
    Ok(format!("{picotool} load -f -x {}", quote(&arquivo)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kinein_protocol::{ProjectArtifacts, ProjectModel, TargetModel};

    use super::FirmwareImage;
    use crate::flash::{FlashError, propose};

    fn modelo(chip: Option<&str>) -> ProjectModel {
        ProjectModel {
            target: TargetModel {
                chip: chip.map(str::to_owned),
                ..TargetModel::default()
            },
            root: "/p".to_owned(),
            embedded: true,
            frameworks: Vec::new(),
            sdks: Vec::new(),
            artifacts: ProjectArtifacts::default(),
            hints: Vec::new(),
        }
    }

    fn imagem(
        engine: &str,
        offset: Option<&str>,
        chip: Option<&str>,
        arquivo: &str,
    ) -> FirmwareImage {
        FirmwareImage {
            label: "MicroPython — teste".to_owned(),
            engine: engine.to_owned(),
            offset: offset.map(str::to_owned),
            chip: chip.map(str::to_owned),
            file: PathBuf::from(arquivo),
            size_bytes: 1_790_544,
        }
    }

    fn acha(binario: &str) -> Option<PathBuf> {
        match binario {
            "esptool" => Some(PathBuf::from("/x/esptool")),
            "picotool" => Some(PathBuf::from("/x/picotool")),
            _ => None,
        }
    }

    /// ESP32 classico: `--chip esp32 … write-flash 0x1000 <bin>`, o aviso do
    /// erase-flash com a porta, o aviso de chip diferente do kit, e a flash
    /// pequena demais.
    #[test]
    fn an_esptool_firmware_writes_the_file_at_the_page_offset() {
        let modelo = modelo(Some("esp32c3"));
        let fw = imagem(
            "esptool",
            Some("0x1000"),
            Some("esp32"),
            "/ide/fw/o meu.bin",
        );
        let p = propose(
            &modelo,
            None,
            Some("/dev/ttyUSB0"),
            Some(1_048_576),
            Some(&fw),
            None,
            &acha,
        )
        .unwrap();
        assert_eq!(
            p.command,
            "'/x/esptool' --chip esp32 --port '/dev/ttyUSB0' --baud 460800 --before default-reset \
             --after hard-reset write-flash 0x1000 '/ide/fw/o meu.bin'"
        );
        assert_eq!(p.engine, "esptool");
        assert!(p.name.contains("firmware"));
        assert!(p.source.iter().any(|s| s.contains("offset: 0x1000")));
        assert!(
            p.warnings
                .iter()
                .any(|w| w.contains("erase-flash") && w.contains("/dev/ttyUSB0"))
        );
        assert!(
            p.warnings
                .iter()
                .any(|w| w.contains("kit diz chip esp32c3"))
        );
        assert!(
            p.warnings.iter().any(|w| w.contains("1MB de flash")),
            "{:?}",
            p.warnings
        );

        // Sem porta: recusa; motor pedido diferente: recusa nomeando a pagina.
        assert!(matches!(
            propose(&modelo, None, None, None, Some(&fw), None, &acha),
            Err(FlashError::NoDevice { .. })
        ));
        assert!(matches!(
            propose(
                &modelo,
                Some("probe-rs"),
                Some("/dev/ttyUSB0"),
                None,
                Some(&fw),
                None,
                &acha
            ),
            Err(FlashError::UnknownEngine { .. })
        ));
        // Sem esptool: ferramenta ausente.
        assert!(matches!(
            propose(
                &modelo,
                None,
                Some("/dev/ttyUSB0"),
                None,
                Some(&fw),
                None,
                &|_| None
            ),
            Err(FlashError::MissingTool { .. })
        ));
    }

    /// Pico: `picotool load -f -x <uf2>`, sem porta, sem offset.
    #[test]
    fn a_picotool_firmware_loads_the_uf2() {
        let fw = imagem("picotool", None, None, "/ide/fw/RPI_PICO.uf2");
        let p = propose(&modelo(None), None, None, None, Some(&fw), None, &acha).unwrap();
        assert_eq!(p.command, "'/x/picotool' load -f -x '/ide/fw/RPI_PICO.uf2'");
        assert!(p.source.iter().any(|s| s.contains("RPI-RP2")));
        assert!(p.warnings.is_empty());
    }
}
