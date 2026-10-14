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
    l.cli_models = "studio-pro: protocolo baseado no código de referência UGREEN Studio Pro HP206 e sua captura\nStatus do teste de hardware: VERIFICADO em firmware 0.2.5 (Linux, canal 1; veja docs/verification.md)\nHiTune Max5c: NÃO COMPATÍVEL (IDs de comando conflitantes)\nOutros modelos/firmware UGREEN: NÃO VERIFICADOS";
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
    // Fragmentos de ajuda gerada; a tela
    // é renderizada a partir do registro de comandos.
    l.help_subtitle = "CLI Bluetooth não oficial para UGREEN Studio Pro";
    l.help_usage = "USO";
    l.help_offline = "COMANDOS OFFLINE";
    l.help_bluetooth = "COMANDOS BLUETOOTH (emparelhe primeiro nas configurações do sistema)";
    l.help_capture_gated = "CONSULTAS COM PORTA (atualmente recusam antes de conectar)";
    l.help_options = "OPÇÕES (antes do COMANDO)";
    l.help_examples = "EXEMPLOS";
    l.help_footer = "Protocolo Studio Pro HP206 verificado no firmware retail 0.2.5 (uma unidade);\nHiTune Max5c usa identificadores de comando conflitantes. Não há ações de\nfirmware, reinício, gravação bruta nem busca de fones. Perfis podem ser aplicados parcialmente em caso de falha.";
    l.opt_address = "Endereço Bluetooth explícito do destino";
    l.opt_model = "Obrigatório para solicitações de hardware; nunca detectado automaticamente";
    l.opt_channel = "Canal RFCOMM, 1–30 (padrão 1)";
    l.opt_timeout = "Tempo limite por operação em segundos, 1–60";
    l.opt_lang = "Idioma da interface: en, es, pt, de, fr, it, nl, ru, zh, ja, ko (padrão en)";
    l.opt_autoconnect = "Conectar ao último dispositivo ao iniciar (padrão ativo)";
    l.opt_no_autoconnect = "Desativar a conexão automática ao iniciar";
    l.opt_json = "JSON versionado para comandos de leitura compatíveis";
    l.opt_dry_run = "Imprimir os pacotes de configurações sem acesso Bluetooth";
    l.opt_help = "Mostrar ajuda";
    l.opt_version = "Mostrar versão";
    l.cmd_help = "Mostrar esta ajuda";
    l.cmd_tui = "Abrir a interface de terminal interativa opcional";
    l.cmd_models = "Mostrar compatibilidade de protocolo e limites";
    l.cmd_commands = "Listar as configurações admitidas";
    l.cmd_decode = "Validar/decodificar bytes de resposta capturados offline";
    l.cmd_analyze = "Interpretar achados estáticos do APK offline";
    l.cmd_candidate = "Imprimir um candidato de pacote estático; nunca é enviado";
    l.cmd_profile = "Gerenciar perfis de configurações";
    l.cmd_profile_example = "Imprimir um perfil de configurações de exemplo";
    l.cmd_profile_export = "Ler as configurações e imprimir um perfil reutilizável";
    l.cmd_profile_apply = "Validar o perfil, aplicar e verificar cada configuração";
    l.cmd_discover = "Listar dispositivos emparelhados em cache; sem varredura de rádio";
    l.cmd_capture = "Gravar os bytes brutos TX/RX de uma sessão como hexadecimal de teste";
    l.cmd_status = "Ler bateria, firmware e configurações";
    l.cmd_set = "Alterar uma configuração e depois verificar a leitura";
    l.cmd_query = "Consulta com porta; não verificada neste firmware";
    l.cmd_control = "Controle com porta; --dry-run imprime os bytes";
    l.cmd_find = "Tom de busca com porta";
    l.host_commands = [
        "Observar transporte de áudio do sistema",
        "Listar saídas de áudio do host",
        "Mostrar evidências das funções",
        "Ler diagnóstico do host",
        "Conectar ligação Bluetooth do host",
        "Desconectar ligação Bluetooth do host",
    ];
    l.cmd_audio_devices = "Listar saídas de áudio do Windows";
    l.features_title = " Recursos ";
    l.features_search = "Buscar: ";
    l.features_all = "Tudo";
    l.feature_categories = [
        "Áudio",
        "Conexão",
        "Ambiente",
        "Respostas",
        "Informação do dispositivo",
        "Multiponto",
        "Códecs",
        "Áudio avançado",
        "Controles",
        "Compartilhamento de áudio",
        "Localizar",
        "Gerenciamento do dispositivo",
        "Diagnóstico",
    ];
    l.feature_names = [
        "ANC",
        "Equalizador",
        "Modo de jogo",
        "Áudio espacial",
        "Conexão dupla",
        "Redução de vento",
        "Avisos de voz",
        "Ação subir volume",
        "Ação descer volume",
        "Dispositivos multiponto",
        "Códec de áudio",
        "ID do fone",
        "Cor do fone",
        "MAC do estojo",
        "Controle LDAC",
        "LDAC para um emparelhado",
        "Decodificação de alta qualidade",
        "Comando de volume",
        "Canal mono/stéreo",
        "Mapeamento de botões",
        "Ações de IA",
        "Controle de IA",
        "Desativar compartilhamento",
        "Compartilhar com um emparelhado",
        "Compartilhar com um emparelhado escolhido",
        "Localizar fone esquerdo",
        "Localizar fone direito",
        "Restauração de fábrica",
        "Decodificador de capturas",
        "Análise de capturas",
        "Prévia de pacote",
    ];
    l.feature_descriptions = [
        "Profundidade de cancelamento ativo de ruído. Ajuste verificado; edite na tela de Ajustes.",
        "Predefinição do equalizador. Ajuste verificado; edite na tela de Ajustes.",
        "Alternador de baixa latência do modo de jogo. Ajuste verificado; edite na tela de Ajustes.",
        "Alternador de áudio espacial. Ajuste verificado; edite na tela de Ajustes.",
        "Alternador de conexão com dois dispositivos. Ajuste verificado; edite na tela de Ajustes.",
        "Alternador de redução de ruído de vento. Ajuste verificado; edite na tela de Ajustes.",
        "Estilo dos avisos de voz. Ajuste verificado; edite na tela de Ajustes.",
        "Ação de segurar subir volume. Ajuste verificado; edite na tela de Ajustes.",
        "Ação de segurar descer volume. Ajuste verificado; edite na tela de Ajustes.",
        "Ler a lista de emparelhados multiponto. O protocolo de emparelhados ainda requer verificação em hardware.",
        "Ler o códec de áudio Bluetooth ativo. O formato de consulta ainda requer verificação em hardware.",
        "Ler o ID de hardware do fone. O formato de consulta ainda requer verificação em hardware.",
        "Ler a cor do fone. O formato de consulta ainda requer verificação em hardware.",
        "Ler o MAC do estojo de carga. O formato de consulta ainda requer verificação em hardware.",
        "Alternar a codificação LDAC na conexão atual. O formato de controle ainda requer verificação em hardware.",
        "Alternar a codificação LDAC para um emparelhado. O formato de controle ainda requer verificação em hardware.",
        "Alternar a decodificação de áudio de alta qualidade. O formato de controle ainda requer verificação em hardware.",
        "Enviar um byte de volume absoluto. A faixa de hardware não está verificada.",
        "Trocar o canal de áudio entre mono e stéreo. O formato de controle ainda requer verificação em hardware.",
        "Remapear as oito ações de segurar. O formato de controle ainda requer verificação em hardware.",
        "Remapear as quatro ações acionadas por IA. O formato de controle ainda requer verificação em hardware.",
        "Alternar o controle de áudio por IA. O formato de controle ainda requer verificação em hardware.",
        "Desativar o compartilhamento de áudio. O formato de controle ainda requer verificação em hardware.",
        "Compartilhar áudio com o único emparelhado conectado. O formato de controle ainda requer verificação em hardware.",
        "Compartilhar áudio com um emparelhado escolhido. O formato de controle ainda requer verificação em hardware.",
        "Reproduzir um tom de localização no fone esquerdo pelo host. O protocolo de localização ainda requer verificação em hardware.",
        "Reproduzir um tom de localização no fone direito pelo host. O protocolo de localização ainda requer verificação em hardware.",
        "Remover o emparelhamento e restaurar os padrões de fábrica. Destrutivo; o formato de controle ainda requer verificação em hardware.",
        "Decodificar bytes de resposta capturados offline; sem acesso Bluetooth.",
        "Interpretar um fluxo de captura completo offline; sem acesso Bluetooth.",
        "Imprimir o pacote estático que um controle enviaria; nunca é transmitido.",
    ];
    l.feature_available = "Disponível";
    l.feature_pending = "Pendente de verificação";
    l.feature_pending_reason =
        "O formato de cabo ainda requer verificação em hardware com o app oficial.";
    l.feature_live_disabled =
        "A execução ao vivo está desativada até haver evidência verificada no dispositivo.";
    l.feature_offline_workflow = "Apenas análise offline; nada é enviado ao fone.";
    l.feature_offline_hint = "Executar: ugreen {}";
    l.feature_kind_setting = "Ajuste verificado";
    l.feature_kind_query = "Consulta de dispositivo";
    l.feature_kind_control = "Controle de dispositivo";
    l.feature_kind_offline = "Análise offline";
    l.feature_category_label = "Categoria: {}";
    l.feature_current = "Atual: {}";
    l.feature_edit_in_settings = "Editar em Ajustes (s)";
    l.feature_run_query = "Enter: executar consulta";
    l.feature_configure = "Enter: configurar";
    l.feature_back = "Esc: voltar";
    l.feature_form_title = " Configurar {} ";
    l.feature_err_required = "Preencha todos os campos antes de revisar";
    l.feature_field_unset = "não definido";
    l.feature_form_hint = "Acima Abaixo campo / Esq Dir opção / Enter revisar / Esc voltar";
    l.feature_review_title = " Revisar {} ";
    l.review_write_title = " Revisar escrita ";
    l.feature_review_target = "Destino: {}";
    l.feature_review_previous = "Atual: {}";
    l.feature_review_proposed = "Proposto: {}";
    l.feature_review_values = "Valores: {}";
    l.feature_review_readback = "Leitura: {}";
    l.feature_readback_verified = "verificado por confirmação e leitura correspondente";
    l.feature_readback_noack =
        "verificado por leitura correspondente (áudio espacial não envia confirmação)";
    l.feature_readback_unconfirmed = "não confirmado de forma independente; atualize após aplicar";
    l.feature_readback_query = "consulta somente leitura";
    l.feature_confirm_apply = "Enter: aplicar    Esc: cancelar";
    l.feature_confirm_reset =
        "Destrutivo: remove o emparelhamento do host e restaura os padrões de fábrica.";
    l.features_footer = [
        [
            ("Acima", "mover"),
            ("Abaixo", "mover"),
            ("Tab", "categoria"),
            ("Enter", "abrir"),
            ("Esc", "voltar"),
            ("", ""),
        ],
        [
            ("letras", "buscar"),
            ("Backspace", "apagar"),
            ("Ctrl+U", "limpar"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "fechar"),
            ("?", "atalhos"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "Executando operação de recursos...";
    l.status_edit_in_settings = "{} é um ajuste verificado; edite na tela de Ajustes (s).";
    l.codec_unverified = "consulta não verificada";
    l.arg_enabled = "Ativado";
    l.arg_side = "Lado";
    l.arg_duration = "Duração (s)";
    l.arg_target = "Alvo";
    l.arg_value = "Valor (0-255)";
    l.arg_mode = "Modo";
    l.arg_peer = "Emparelhado";
    l.arg_actions = "Ações";
    l.arg_address = "MAC do emparelhado";
    l.arg_hint_toggle = "on ou off";
    l.arg_hint_side = "esquerda ou direita";
    l.arg_hint_seconds = "1..30 segundos";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "seis octetos hexadecimais separados por dois pontos";
    l.arg_hint_peer = "1 ou 2";
    l.arg_hint_actions = "{} valores de ação separados por vírgulas";
    l.tui_no_tty =
        "A interface exige terminais interativas de entrada e saída; execute ugreen --help para os comandos da CLI.";
    l.tui_bad_channel =
        "O canal da interface deve ser 1-30 e o tempo de espera maior que zero e no máximo 60 segundos.";
    l.status_worker_busy =
        "Já há uma operação em execução; aguarde ou pressione Esc para cancelar.";
    l.status_worker_sequence = "Sequência de solicitações esgotada; reinicie a interface.";
    l.status_worker_stopped = "O worker Bluetooth parou; reinicie a interface.";
    l.status_worker_queue = "A fila do worker está cheia; aguarde antes de tentar novamente.";
    l.status_reset_unavailable = "alvo de restauração indisponível";
    l.status_reset_done =
        "Restauração confirmada; emparelhamento do host e dispositivo salvo removidos.";
    l.status_inflight_write =
        "Uma gravação em andamento pode ter sido concluída; atualize após reiniciar.";
    l
}
