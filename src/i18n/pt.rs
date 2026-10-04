//! Portuguese catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / controle Studio Pro ";
    l.connected = "CONECTADO";
    l.disconnected = "DESCONECTADO";
    l.unknown = "DESCONHECIDO";
    l.target = "Destino: ";
    l.target_none = "não escolhido";
    l.protocol = "Protocolo: ";
    l.protocol_on = "Studio Pro HP206 (escolhido por você)";
    l.protocol_off = "não escolhido; m para confirmar Studio Pro HP206";
    l.battery = "Bateria: ";
    l.battery_pct = "{}%";
    l.unavailable = "indisponível";
    l.codec_line = "Codec: não detectado";
    l.firmware = "Firmware: ";
    l.snapshot = "Leitura: ";
    l.snapshot_stale = "DESATUALIZADA / não atual";
    l.snapshot_ago = "lida há {}s";
    l.snapshot_never = "não lida";
    l.rfcomm_line = "RFCOMM {} | tempo por operação {}s | fw 0.2.5 verificado";
    l.labels = [
        "ANC",
        "Equalizador",
        "Modo jogo",
        "Áudio espacial",
        "Conexão dupla",
        "Redução de vento",
        "Avisos",
        "Ação subir volume",
        "Ação descer volume",
    ];
    l.proposed = "  -> {} (proposto)";
    l.settings_title = " Ajustes / leitura do dispositivo, sem escrita automática ";
    l.settings_blocked = " Ajustes / ESCRITA BLOQUEADA: r para atualizar ";
    l.settings_stale = " Ajustes / última leitura desatualizada ";
    l.status_title = " Estado ";
    l.status_working = " Trabalhando / Esc cancela as etapas seguintes ";
    l.log_title = " Registro / Cima Baixo Início Fim / l Esc fechar ";
    l.shortcuts_title = " Atalhos / Cima Baixo para rolar / ? Esc fechar ";
    l.edit_title = " Editar endereço de destino ";
    l.edit_lines = &[
        "Digite dígitos hexadecimais e dois pontos. Backspace apaga; Ctrl-U limpa.",
        "Enter salva localmente; Esc cancela. Nenhuma conexão é iniciada.",
    ];
    l.model_title = " Confirmar protocolo específico do modelo ";
    l.model_lines = &[
        "Use apenas com UGREEN Studio Pro HP206.",
        "Verificado em firmware 0.2.5 (uma unidade); outras",
        "variantes e firmware não estão verificados.",
        "HiTune Max5c usa IDs de comando conflitantes e não é compatível.",
        "Isto escolhe um protocolo; não prova a identidade do dispositivo.",
        "y / Enter: escolher Studio Pro HP206    n / Esc: cancelar",
    ];
    l.paired_title = " Emparelhados / Cima Baixo Enter escolher / Esc fechar ";
    l.paired_empty = "Nenhum dispositivo emparelhado em cache. Esc fecha.";
    l.quit_title = " Sair com trabalho em andamento? ";
    l.quit_lines = &[
        "Cancelamento pedido; etapas seguintes serão ignoradas.",
        "Uma escrita em andamento pode ter sido concluída.",
        "Sair nunca espera o worker.",
        "q / Enter: sair já    Esc: ficar e esperar o resultado",
    ];
    l.resize_to = "Redimensione para pelo menos {}x{}";
    l.resize_quit = "Esc: cancelar  q: sair";
    l.resize_inflight = "Uma escrita pode ter sido concluída. q/Enter: sair; Esc: ficar";
    l.resize_disabled = "Ações desativadas neste tamanho.";
    l.footer = [
        [
            ("a", "endereço"),
            ("p", "dispositivos"),
            ("m", "modelo"),
            ("c", "conectar"),
            ("r", "atualizar"),
            ("d", "desconectar"),
        ],
        [
            ("<", "menos"),
            (">", "mais"),
            ("Enter", "aplicar"),
            ("y", "confirmar"),
            ("Esc", "voltar"),
            ("", ""),
        ],
        [
            ("l", "registro"),
            ("?", "atalhos"),
            ("L", "idioma"),
            ("q", "sair"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Desconectado. Sem acesso Bluetooth ainda. Edite o endereço (a) ou leia os emparelhados (p).";
    l.status_loading = "Carregando dispositivos emparelhados em cache (sem varredura)...";
    l.status_connecting = "Conectando e lendo estado e firmware...";
    l.status_refreshing = "Atualizando estado e firmware...";
    l.status_disconnecting = "Desconectando o controle...";
    l.status_applying =
        "Aplicando {}={} após consulta prévia, verificando confirmação e leitura...";
    l.status_paired_empty =
        "Sem dispositivos emparelhados em cache. Emparelhe nas configurações de Bluetooth primeiro.";
    l.status_paired_pick =
        "Selecione o dispositivo pretendido e pressione Enter. Selecionar não conecta.";
    l.status_read_ok =
        "Estado lido com sucesso. Os ajustes mostrados são a leitura do dispositivo.";
    l.status_firmware_note = "Ajustes lidos; firmware indisponível: {}";
    l.status_blocked_note = " Atualização explícita (r) ainda é exigida para desbloquear escritas.";
    l.status_verified = "Verificado {}={} por confirmação e leitura correspondente do dispositivo.";
    l.status_disconnected = "Controle desconectado. O Bluetooth não foi modificado.";
    l.status_quit_confirm = "Cancelamento pedido. Uma escrita em andamento pode ter sido concluída. Pressione q ou Enter para sair sem esperar; Esc para ficar.";
    l.status_cancel_busy = "Cancelamento pedido; aguardando a etapa atual. Uma escrita em andamento pode ter sido concluída. Etapas seguintes serão ignoradas.";
    l.status_address_saved = "Endereço salvo localmente. Pressione c para conectar explicitamente.";
    l.status_bad_address =
        "Digite seis octetos hexadecimais separados por dois pontos, ex. AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Protocolo Studio Pro HP206 escolhido por você, sem verificação de identidade do dispositivo.";
    l.status_device_picked =
        "Endereço escolhido. Confirme o protocolo Studio Pro para conectar (y ou Enter).";
    l.status_device_bad = "Dispositivo em cache tem endereço inválido; edite manualmente.";
    l.status_cancel_then_disconnect =
        "Cancelamento pedido; a desconexão seguirá a etapa atual. Uma escrita em andamento pode ter sido concluída.";
    l.status_must_disconnect = "Desconecte (d) antes de mudar destino ou protocolo.";
    l.status_must_connect = "Desconectado; conecte (c) antes de atualizar.";
    l.status_proposed = "Apenas proposta. Enter aplica; Esc descarta.";
    l.status_unavailable_setting =
        "Este ajuste não está disponível na leitura do dispositivo; escrita desativada.";
    l.status_already_matches =
        "Esse valor já coincide com a última leitura; não é preciso escrever.";
    l.status_pick_first = "Use Esquerda/Direita para escolher um valor proposto antes de aplicar.";
    l.status_disabled =
        "Alterações desativadas. Conecte e atualize explicitamente após qualquer operação incerta.";
    l.status_lang = "Idioma: {}. Pressione L para alternar.";
    l.err_gone = "Desconectado; conecte primeiro.";
    l.err_status_query = "Falha na consulta de estado: {}";
    l.err_paired_list = "Falha na lista de emparelhados em cache: {}";
    l.err_need_model = "Confirme o protocolo Studio Pro HP206 antes de conectar.";
    l.err_connected_already = "Desconecte antes de conectar outro destino.";
    l.err_connect = "Falha na conexão: {}";
    l.err_uncertain_blocked =
        "Escritas bloqueadas após operação incerta. Atualize explicitamente primeiro.";
    l.err_preflight = "Falha no status de consulta prévia; nenhum ajuste enviado: {}";
    l.err_preflight_unreadable = "A consulta prévia não lê {}; nada enviado.";
    l.err_ack = "O ajuste pode ter sido enviado; falha na confirmação: {}. Atualize explicitamente antes de outra escrita.";
    l.err_readback = "Escrita confirmada, mas falha na leitura: {}. Atualize explicitamente antes de outra escrita.";
    l.err_readback_noack = "Escrita enviada, mas falha na leitura: {}. Atualize explicitamente antes de outra escrita.";
    l.err_mismatch = "Escrita confirmada, mas {} não correspondeu a {}. Atualize explicitamente antes de outra escrita.";
    l.err_mismatch_noack = "Escrita enviada, mas {} não correspondeu a {}. Atualize explicitamente antes de outra escrita.";
    l.err_cancel_uncertain = "Etapas seguintes canceladas. Uma escrita em andamento pode ter sido concluída; atualize explicitamente antes de outra escrita.";
    l.err_cancel = "Cancelado; nenhuma etapa seguinte iniciará.";
    l.cli_help = "ugreen 0.1.0 — CLI Bluetooth não oficial para UGREEN Studio Pro\n\nUSO\n  ugreen [OPÇÕES] COMANDO\n\nCOMANDOS OFFLINE\n  help                          Mostrar esta ajuda\n  tui                           Abrir a interface de terminal opcional\n  models                        Mostrar compatibilidade e limites\n  commands                      Listar ajustes suportados\n  decode HEX                    Validar/decodificar bytes capturados offline\n  profile example               Mostrar um perfil de exemplo\n\nCOMANDOS BLUETOOTH (emparelhe nas configurações do sistema)\n  discover                      Listar emparelhados em cache; sem varredura de rádio\n  capture                       Gravar os bytes TX/RX brutos de uma sessão como hex\n  status                        Ler bateria, firmware e ajustes\n  set CHAVE VALOR               Mudar um ajuste e verificar a leitura\n  profile export                Ler ajustes e imprimir um perfil reutilizável\n  profile apply ARQUIVO         Validar perfil, aplicar e verificar cada ajuste\n\nOPÇÕES (antes do COMANDO)\n  --address XX:XX:XX:XX:XX:XX    Endereço Bluetooth de destino explícito\n  --model studio-pro            Obrigatório para requisições de hardware; nunca detectado\n  --channel 1                   Canal RFCOMM, 1–30 (padrão 1)\n  --timeout 3                   Tempo por operação em segundos, 1–60\n  --lang pt                     Idioma da interface: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (padrão en)\n  --autoconnect                 A TUI conecta ao destino memorizado ao iniciar\n  --no-autoconnect              Desativa a conexão automática ao iniciar\n  --dry-run                     Mostrar pacotes de ajustes sem acesso Bluetooth\n  --help, -h                    Mostrar ajuda\n  --version, -V                 Mostrar versão\n\nEXEMPLOS\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > meu-perfil.conf\n  ugreen --dry-run profile apply meu-perfil.conf\n\nProtocolo Studio Pro HP206 verificado em firmware 0.2.5 (uma unidade);\nHiTune Max5c usa IDs de comando conflitantes. Sem firmware, restauração,\nescrita livre ou localização. Perfis podem ser aplicados parcialmente.\n";
    l.cli_models = "studio-pro: protocolo baseado no código de referência UGREEN Studio Pro HP206 e sua captura\nStatus do teste de hardware: VERIFICADO em firmware 0.2.5 (Linux, canal 1; veja docs/verification.md)\nHiTune Max5c: NÃO COMPATÍVEL (IDs de comando conflitantes)\nOutros modelos/firmware UGREEN: NÃO VERIFICADOS";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Usando protocolo Studio Pro HP206 em {}, canal RFCOMM {}";
    l.cli_need_model =
        "passe --model studio-pro para escolher explicitamente este protocolo específico";
    l.cli_need_address = "falta --address; use discover para listar emparelhados";
    l.cli_connect_fail = "Falha na conexão Bluetooth: {}. Verifique emparelhamento, energia, adaptador e se outro app usa o canal de controle";
    l.cli_preflight_fail = "falha na consulta prévia; ajustes não enviados: {}";
    l.cli_verified = "Verificado {}={}";
    l.cli_apply_fail = "{} de {} ajustes verificados antes da falha em {}={}: {}. Ajustes anteriores não são revertidos";
    l.cli_model_note = "model=studio-pro (escolhido por você; não é verificação de identidade)";
    l.cli_unknown_value = "desconhecido";
    l.cli_firmware_fail = "ajustes lidos, mas falha na consulta de firmware: {}";
    l.cli_no_paired =
        "Nenhum dispositivo emparelhado retornado. Emparelhe os fones nas configurações de Bluetooth primeiro.";
    l.cli_paired_fail = "falha na listagem de emparelhados: {}";
    l.cli_dry_run_writes = "Simulação: sem conexão Bluetooth nem escritas no dispositivo";
    l.cli_bad_model = "apenas --model studio-pro está implementado; IDs do Max5c são incompatíveis";
    l.cli_bad_option = "opção desconhecida '{}'";
    l.cli_bad_channel = "o canal deve ser 1–30";
    l.cli_bad_timeout = "o tempo deve ser 1–60 segundos";
    l.cli_need_value = "{} requer um valor";
    l.cli_unknown_command = "comando desconhecido '{}'; execute ugreen --help";
    l.cli_bad_usage = "uso: {}";
    l.cli_bad_decode = "a captura não é um fluxo de resposta limpo e completo: {} quadros, {} CRCs rejeitados, {} bytes descartados, {} quadros desconhecidos, {} bytes pendentes";
    l.cli_no_tty = "A TUI exige terminais interativos em stdin e stdout; execute ugreen --help para comandos CLI.";
    l.cli_no_tui_feature =
        "A TUI está desativada nesta compilação só-CLI; recompile sem --no-default-features";
    l.cli_tui_dry =
        "tui não pode ser combinado com --dry-run; use --dry-run set CHAVE VALOR para pacotes offline";
    l.cli_bad_profile = "o perfil excede 16 KiB";
    l.cli_inflight_after_quit = "Uma escrita em andamento pode ter sido concluída. Atualize explicitamente o status do dispositivo antes de novas mudanças.";
    l.set_bad_anc = "valor ANC inválido";
    l.set_bad_eq = "valor EQ inválido";
    l.set_bad_onoff = "esperado on ou off";
    l.set_bad_prompts = "esperado voice ou beeps";
    l.set_bad_button = "esperado none, next ou previous";
    l.set_unknown = "ajuste desconhecido '{}'\n{}";
    l.profile_big = "o perfil excede 16 KiB";
    l.profile_line = "linha {} do perfil deve ser chave=valor";
    l.profile_dup = "chave duplicada '{}'";
    l.profile_model = "o modelo do perfil deve ser studio-pro";
    l.profile_need_model = "o perfil requer model=studio-pro";
    l.profile_empty = "o perfil não tem ajustes";
    l.req_timeout = "nenhuma resposta válida antes do prazo";
    l.req_closed = "conexão Bluetooth fechada";
    l.req_rejected = "os fones rejeitaram a instrução 0x{:02X}";
    l.info_short = "carga de informações do dispositivo muito curta: {} bytes (mínimo 8)";
    l.fw_short = "resposta de firmware não traz bytes de versão";
    l.set_ackfail = "o ajuste pode ter sido enviado, mas falha na confirmação: {}; consulte o status antes de retentear";
    l.set_rbfail = "escrita confirmada, mas falha na leitura: {}";
    l.set_rbfail_noack = "escrita enviada, mas falha na leitura: {}";
    l.set_mismatch =
        "escrita confirmada, mas a leitura de {} não corresponde a {}; consulte o estado";
    l.set_mismatch_noack =
        "escrita enviada, mas a leitura de {} não corresponde a {}; consulte o estado";
    l
}
