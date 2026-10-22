//! Gravar um FIRMWARE baixado (C5 do `roadmaps/41` bloco C): a linha que a
//! pagina da placa no micropython.org manda, composta com as pecas do E4.
//!
//! ```text
//! esptool    `esptool [--chip <chip>] --port <porta> --baud 460800 --before default-reset
//!            --after hard-reset write-flash <offset> <arquivo>` — a pagina diz
//!            `esptool.py --port PORTNAME --baud 460800 write_flash 0x1000 <bin>` (ESP32
//!            classico) e `write_flash 0` (C3/S3), lida em 2026-09-17; `write-flash`
//!            e' a grafia v5; a previa adapta flags e resets a versao 4/5
//!            confirmada no binario selecionado (0.186.0)
//! picotool   `picotool load -f -x <uf2>` — a pagina manda copiar o .uf2 para o drive
//!            RPI-RP2 (BOOTSEL); o picotool faz o mesmo pelo USB (E4)
//! ```
//!
//! A pagina manda apagar a flash na primeira instalacao (`esptool.py
//! erase_flash`): vai em `warnings`, com a linha pronta — nunca na linha
//! de gravar, porque apagar e' outra decisao.

use std::path::PathBuf;

use kinein_protocol::ProjectModel;

use super::{
    ESPTOOL_BAUD, FindTool, FlashCommand, FlashError, FlashPlan, ProposalTools, chosen_port, quote,
    readable_size, tool,
};

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
    /// Original catalogue checksum, required by native validation.
    pub expected_sha256: Option<String>,
}

/// A proposta para um firmware: o motor e' o da pagina; um motor pedido
/// diferente e' recusado (a pagina nao da' outra forma).
pub(super) fn propose(
    model: &ProjectModel,
    image: &FirmwareImage,
    engine: Option<&str>,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    tools: &ProposalTools<'_>,
) -> Result<FlashPlan, FlashError> {
    let find_tool = tools.find;
    let (_, engine_source) = select_engine(image, engine)?;
    let mut source = vec![engine_source];
    let mut warnings = Vec::new();
    let command = match image.engine.as_str() {
        "esptool" => esptool(
            image,
            model,
            device,
            flash_size_bytes,
            tools,
            &mut source,
            &mut warnings,
        )?,
        "picotool" => picotool(image, find_tool, &mut source)?,
        outro => {
            return Err(FlashError::UnknownEngine {
                engine: outro.to_owned(),
            });
        }
    };
    Ok(FlashPlan::new(
        format!("Gravar firmware ({})", image.label),
        image.engine.clone(),
        source,
        warnings,
        command,
    ))
}

pub(super) fn select_engine(
    image: &FirmwareImage,
    engine: Option<&str>,
) -> Result<(String, String), FlashError> {
    if let Some(pedido) = engine.map(str::trim).filter(|e| !e.is_empty())
        && pedido != image.engine
    {
        return Err(FlashError::UnknownEngine {
            engine: format!(
                "{pedido} (o firmware {} grava-se por {}, como a página da placa manda)",
                image.label, image.engine
            ),
        });
    }
    Ok((
        image.engine.clone(),
        format!(
            "motor: {} (a página do firmware no micropython.org)",
            image.engine
        ),
    ))
}

fn esptool(
    image: &FirmwareImage,
    model: &ProjectModel,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    tools: &ProposalTools<'_>,
    source: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
    let selected = super::esptool::selected(tools)?;
    source.push(selected.evidence());
    let device = chosen_port(device)?;
    let offset = image
        .offset
        .as_deref()
        .ok_or_else(|| FlashError::NoArtifact {
            message: format!("o catalogo não diz o offset de {}", image.label),
        })?;
    let arquivo = image.file.display().to_string();
    let mut partes = Vec::new();
    // O chip da pagina; se o kit diz outro, o usuario ve os dois nos avisos
    // e a linha leva o da pagina (e' o firmware que decide o chip).
    if let Some(chip) = image.chip.as_deref() {
        partes.push("--chip".to_owned());
        partes.push(chip.to_owned());
        if let Some(kit) = model.target.chip.as_deref().filter(|k| *k != chip) {
            warnings.push(format!(
                "o kit diz chip {kit} e este firmware é para {chip}: confira a placa antes de gravar"
            ));
        }
    }
    partes.extend([
        "--port".to_owned(),
        device.to_owned(),
        "--baud".to_owned(),
        ESPTOOL_BAUD.to_owned(),
        "--before".to_owned(),
        selected.syntax("default-reset"),
        "--after".to_owned(),
        selected.syntax("hard-reset"),
        selected.syntax("write-flash"),
        offset.to_owned(),
        arquivo.clone(),
    ]);
    source.push(format!(
        "firmware: {arquivo} ({})",
        readable_size(image.size_bytes)
    ));
    source.push(format!(
        "offset: {offset} (a página da placa no micropython.org)"
    ));
    source.push(format!(
        "porta: {device} (escolhida no painel de Embarcados)"
    ));
    warnings.push(format!(
        "primeira instalação do MicroPython nesta placa? a página manda apagar a flash antes: \
         {} --port {} {}",
        quote(&selected.path.display().to_string()),
        quote(device),
        selected.syntax("erase-flash")
    ));
    if let Some(placa) = flash_size_bytes
        && image.size_bytes > placa
    {
        warnings.push(format!(
            "o firmware tem {} e a placa identificada tem {} de flash",
            readable_size(image.size_bytes),
            readable_size(placa)
        ));
    }
    Ok(FlashCommand::new(selected.path.clone(), partes))
}

fn picotool(
    image: &FirmwareImage,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
    let picotool = tool(
        find_tool,
        "picotool",
        "pacote `picotool` da distro ou o build oficial (github.com/raspberrypi/picotool)",
    )?;
    let arquivo = image.file.display().to_string();
    source.push(format!(
        "firmware: {arquivo} ({})",
        readable_size(image.size_bytes)
    ));
    source.push(
        "modo: -f forca o BOOTSEL pelo USB; -x executa depois — o mesmo que copiar o .uf2 \
         para o drive RPI-RP2, como a página manda"
            .to_owned(),
    );
    Ok(FlashCommand::new(
        picotool,
        vec!["load".to_owned(), "-f".to_owned(), "-x".to_owned(), arquivo],
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kinein_protocol::{ProjectArtifacts, ProjectModel, TargetModel};

    use super::FirmwareImage;
    use crate::flash::{FlashError, propose_v5 as propose};

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

    fn image(
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
            expected_sha256: None,
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
        let fw = image(
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
        let fw = image("picotool", None, None, "/ide/fw/RPI_PICO.uf2");
        let p = propose(&modelo(None), None, None, None, Some(&fw), None, &acha).unwrap();
        assert_eq!(p.command, "'/x/picotool' load -f -x '/ide/fw/RPI_PICO.uf2'");
        assert!(p.source.iter().any(|s| s.contains("RPI-RP2")));
        assert!(p.warnings.is_empty());
    }
}
