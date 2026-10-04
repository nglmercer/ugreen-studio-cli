//! French catalog. Starts from English; untranslated strings
//! stay in English. `{}` placeholders keep the same order.
use super::L;

pub fn catalog() -> L {
    let mut l = super::en::catalog();
    l.app_title = " UGREEN / contrôle Studio Pro ";
    l.connected = "CONNECTÉ";
    l.disconnected = "DÉCONNECTÉ";
    l.unknown = "INCONNU";
    l.target = "Cible : ";
    l.target_none = "non sélectionnée";
    l.protocol = "Protocole : ";
    l.protocol_on = "Studio Pro HP206 (choisi par vous)";
    l.protocol_off = "non sélectionné ; m pour confirmer Studio Pro HP206";
    l.battery = "Batterie : ";
    l.battery_pct = "{} %";
    l.unavailable = "indisponible";
    l.codec_line = "Codec : non détecté";
    l.firmware = "Firmware : ";
    l.snapshot = "Relevé : ";
    l.snapshot_stale = "PÉRIMÉ / non à jour";
    l.snapshot_ago = "relevé il y a {} s";
    l.snapshot_never = "non relevé";
    l.rfcomm_line = "RFCOMM {} | délai par opération {} s | fw 0.2.5 vérifié";
    l.labels = [
        "ANC",
        "Égaliseur",
        "Mode jeu",
        "Audio spatial",
        "Double connexion",
        "Réduction du vent",
        "Annonces",
        "Action montée volume",
        "Action descente volume",
    ];
    l.proposed = "  -> {} (proposé)";
    l.settings_title = " Réglages / relevé de l'appareil, aucune écriture automatique ";
    l.settings_blocked = " Réglages / ÉCRITURES BLOQUÉES : r pour actualiser ";
    l.settings_stale = " Réglages / dernier relevé périmé ";
    l.status_title = " État ";
    l.status_working = " Travail / Échap annule les étapes suivantes ";
    l.log_title = " Journal / Haut Bas Début Fin / l Échap fermer ";
    l.shortcuts_title = " Raccourcis / Haut Bas défile / ? Échap fermer ";
    l.edit_title = " Éditer l'adresse cible ";
    l.edit_lines = &[
        "Tapez des chiffres hexadécimaux et des deux-points. Retour arrière efface ; Ctrl-U vide.",
        "Entrée enregistre localement ; Échap annule. Aucune connexion n'est lancée.",
    ];
    l.model_title = " Confirmer le protocole spécifique au modèle ";
    l.model_lines = &[
        "À utiliser uniquement avec UGREEN Studio Pro HP206.",
        "Vérifié sur le firmware retail 0.2.5 (une unité) ; autres",
        "variantes et firmwares non vérifiés.",
        "HiTune Max5c utilise des identifiants de commande conflictuels et n'est pas pris en charge.",
        "Ceci choisit un protocole ; il ne prouve pas l'identité de l'appareil.",
        "y / Entrée : choisir Studio Pro HP206    n / Échap : annuler",
    ];
    l.paired_title = " Appairages / Haut Bas Entrée choisir / Échap fermer ";
    l.paired_empty = "Aucun appareil appairé en cache. Échap ferme.";
    l.quit_title = " Quitter pendant un travail ? ";
    l.quit_lines = &[
        "Annulation demandée ; les étapes suivantes seront ignorées.",
        "Une écriture en cours peut encore se terminer.",
        "Quitter n'attend jamais le worker.",
        "q / Entrée : quitter maintenant    Échap : rester et attendre le résultat",
    ];
    l.resize_to = "Agrandir à au moins {}x{}";
    l.resize_quit = "Échap : annuler  q : quitter";
    l.resize_inflight =
        "Une écriture en cours peut se terminer. q/Entrée : quitter ; Échap : rester";
    l.resize_disabled = "Actions désactivées à cette taille.";
    l.footer = [
        [
            ("a", "adresse"),
            ("p", "appareils"),
            ("m", "modèle"),
            ("c", "connecter"),
            ("r", "actualiser"),
            ("d", "déconnecter"),
        ],
        [
            ("<", "moins"),
            (">", "plus"),
            ("Entrée", "appliquer"),
            ("y", "confirmer"),
            ("Échap", "retour"),
            ("", ""),
        ],
        [
            ("l", "journal"),
            ("?", "raccourcis"),
            ("L", "langue"),
            ("q", "quitter"),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_startup =
        "Déconnecté. Pas encore d'accès Bluetooth. Éditez l'adresse (a) ou lisez le cache (p).";
    l.status_loading = "Chargement des appareils appairés en cache (aucun scan)...";
    l.status_connecting = "Connexion, puis lecture de l'état et du firmware...";
    l.status_refreshing = "Actualisation de l'état et du firmware...";
    l.status_disconnecting = "Déconnexion du contrôle...";
    l.status_applying =
        "Application de {}={} après pré-vol, puis vérification de la confirmation et du relevé...";
    l.status_paired_empty =
        "Aucun appareil appairé en cache. Appelez d'abord dans les réglages Bluetooth du système.";
    l.status_paired_pick =
        "Sélectionnez l'appareil voulu et appuyez sur Entrée. Sélectionner ne connecte pas.";
    l.status_read_ok =
        "État relevé avec succès. Les réglages affichés sont le relevé de l'appareil.";
    l.status_firmware_note = "Réglages relevés ; firmware indisponible : {}";
    l.status_blocked_note =
        " Une actualisation explicite (r) est encore requise pour débloquer les écritures.";
    l.status_verified = "{}={} vérifié par confirmation et relevé d'appareil correspondant.";
    l.status_disconnected = "Contrôle déconnecté. Bluetooth n'a pas été modifié.";
    l.status_quit_confirm = "Annulation demandée. Une écriture en cours peut se terminer. Appuyez sur q ou Entrée pour quitter sans attendre ; Échap pour rester.";
    l.status_cancel_busy = "Annulation demandée ; attente de l'étape courante. Une écriture en cours peut se terminer. Les étapes suivantes seront ignorées.";
    l.status_address_saved =
        "Adresse enregistrée localement. Appuyez sur c pour connecter explicitement.";
    l.status_bad_address =
        "Saisissez six octets hexadécimaux séparés par deux-points, ex. AA:BB:CC:DD:EE:FF.";
    l.status_model_on =
        "Protocole Studio Pro HP206 choisi par vous, identité de l'appareil non vérifiée.";
    l.status_device_picked =
        "Adresse choisie. Confirmez le protocole Studio Pro pour connecter (y ou Entrée).";
    l.status_device_bad = "L'appareil en cache a une adresse invalide ; éditez-la à la main.";
    l.status_cancel_then_disconnect =
        "Annulation demandée ; la déconnexion suivra l'étape courante. Une écriture en cours peut se terminer.";
    l.status_must_disconnect = "Déconnectez (d) avant de changer de cible ou de protocole.";
    l.status_must_connect = "Déconnecté ; connectez (c) avant d'actualiser.";
    l.status_proposed = "Valeur seulement proposée. Entrée applique ; Échap rejette.";
    l.status_unavailable_setting =
        "Ce réglage est absent du relevé de l'appareil ; l'écriture est désactivée.";
    l.status_already_matches =
        "Cette valeur correspond déjà au dernier relevé ; aucune écriture nécessaire.";
    l.status_pick_first =
        "Utilisez Gauche/Droite pour choisir une valeur proposée avant d'appliquer.";
    l.status_disabled =
        "Modifications désactivées. Connectez et actualisez explicitement après toute opération incertaine.";
    l.status_lang = "Langue : {}. Appuyez sur L pour parcourir.";
    l.err_gone = "Déconnecté ; connectez d'abord.";
    l.err_status_query = "Échec de la requête d'état : {}";
    l.err_paired_list = "Échec de la liste des appairages en cache : {}";
    l.err_need_model = "Confirmez le protocole Studio Pro HP206 avant de connecter.";
    l.err_connected_already = "Déconnectez avant de connecter une autre cible.";
    l.err_connect = "Échec de la connexion : {}";
    l.err_uncertain_blocked =
        "Écritures bloquées après une opération incertaine. Actualisez explicitement d'abord.";
    l.err_preflight = "Échec de l'état de pré-vol ; aucun réglage envoyé : {}";
    l.err_preflight_unreadable = "Le pré-vol ne lit pas {} ; rien d'envoyé.";
    l.err_ack = "Le réglage a peut-être été envoyé ; échec de la confirmation : {}. Actualisez explicitement avant une autre écriture.";
    l.err_readback = "Écriture confirmée mais le relevé a échoué : {}. Actualisez explicitement avant une autre écriture.";
    l.err_readback_noack = "Écriture envoyée mais le relevé a échoué : {}. Actualisez explicitement avant une autre écriture.";
    l.err_mismatch = "Écriture confirmée mais {} ne correspondait pas à {}. Actualisez explicitement avant une autre écriture.";
    l.err_mismatch_noack = "Écriture envoyée mais {} ne correspondait pas à {}. Actualisez explicitement avant une autre écriture.";
    l.err_cancel_uncertain = "Étapes suivantes annulées. Une écriture en cours a peut-être abouti ; actualisez explicitement avant une autre écriture.";
    l.err_cancel = "Annulé ; aucune étape suivante ne démarrera.";
    l.cli_help = "ugreen 0.1.0 — CLI Bluetooth non officielle pour UGREEN Studio Pro\n\nUTILISATION\n  ugreen [OPTIONS] COMMANDE\n\nCOMMANDES HORS LIGNE\n  help                          Afficher cette aide\n  tui                           Ouvrir l'interface terminale optionnelle\n  models                        Afficher compatibilité et limites du protocole\n  commands                      Lister les réglages pris en charge\n  decode HEX                    Valider/décoder des octets de réponse capturés hors ligne\n  profile example               Afficher un profil de réglages d'exemple\n\nCOMMANDES BLUETOOTH (appelez d'abord dans les réglages système)\n  discover                      Lister les appareils appairés en cache ; aucun scan radio\n  capture                       Enregistrer les octets TX/RX bruts d'une session en hex\n  status                        Lire batterie, firmware et réglages\n  set CLÉ VALEUR                Changer un réglage puis vérifier le relevé\n  profile export                Lire les réglages et imprimer un profil réutilisable\n  profile apply FICHIER         Valider le profil, appliquer et vérifier chaque réglage\n\nOPTIONS (avant la COMMANDE)\n  --address XX:XX:XX:XX:XX:XX    Adresse Bluetooth cible explicite\n  --model studio-pro            Requis pour les requêtes matérielles ; jamais détecté\n  --channel 1                   Canal RFCOMM, 1–30 (défaut 1)\n  --timeout 3                   Délai par opération en secondes, 1–60\n  --lang fr                     Langue de l'interface : en, es, pt, de, fr, it, nl, ru, zh, ja, ko (défaut en)\n  --autoconnect                 Se connecter au dernier appareil au démarrage (activé par défaut)\n  --no-autoconnect              Désactive la connexion automatique au démarrage\n  --dry-run                     Afficher les paquets de réglages sans accès Bluetooth\n  --help, -h                    Afficher l'aide\n  --version, -V                 Afficher la version\n\nEXEMPLES\n  ugreen discover\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro status\n  ugreen --dry-run set anc ultra\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro set eq bass\n  ugreen --address AA:BB:CC:DD:EE:FF --model studio-pro profile export > mon-profil.conf\n  ugreen --dry-run profile apply mon-profil.conf\n\nProtocole Studio Pro HP206 vérifié sur le firmware retail 0.2.5 (une unité) ;\nHiTune Max5c utilise des identifiants de commande conflictuels. Aucune action\nde firmware, réinitialisation, écriture brute ou recherche. Les profils peuvent\ns'appliquer partiellement en cas d'échec.\n";
    l.cli_models = "studio-pro : protocole basé sur le code de référence UGREEN Studio Pro HP206 et sa capture\nStatut du test matériel : VÉRIFIÉ sur le firmware retail 0.2.5 (Linux, canal 1 ; voir docs/verification.md)\nHiTune Max5c : NON PRISE EN CHARGE (identifiants de commande conflictuels)\nAutres modèles/firmwares UGREEN : NON VÉRIFIÉS";
    l.cli_commands = "anc: off|ultra|general|gentle|adaptive|ambient\neq: classic|jazz|electronic|pop|classical|rock|bass|treble\ngame, spatial, dual, wind: on|off\nprompts: voice|beeps\nvolume-up-action, volume-down-action: none|next|previous";
    l.cli_banner = "Protocole Studio Pro HP206 sur {}, canal RFCOMM {}";
    l.cli_need_model =
        "passez --model studio-pro pour choisir explicitement ce protocole spécifique au modèle";
    l.cli_need_address =
        "--address est requis ; utilisez discover pour lister les appareils appairés";
    l.cli_connect_fail = "Échec de la connexion Bluetooth : {}. Vérifiez l'appairage, l'alimentation, l'adaptateur et si une autre application possède le canal de contrôle";
    l.cli_preflight_fail = "échec de l'état de pré-vol ; aucun réglage envoyé : {}";
    l.cli_verified = "Vérifié {}={}";
    l.cli_apply_fail = "{} réglages sur {} vérifiés avant l'échec à {}={} : {}. Les réglages précédents ne sont pas annulés";
    l.cli_model_note =
        "model=studio-pro (choisi par vous ; pas une vérification d'identité d'appareil)";
    l.cli_unknown_value = "inconnu";
    l.cli_firmware_fail = "réglages relevés, mais échec de la requête firmware : {}";
    l.cli_no_paired =
        "Aucun appareil appairé retourné. Appelez les écouteurs d'abord dans les réglages Bluetooth.";
    l.cli_paired_fail = "échec du listage des appareils appairés : {}";
    l.cli_dry_run_writes = "Essai à blanc : aucune connexion Bluetooth ni écriture sur l'appareil";
    l.cli_bad_model =
        "seul --model studio-pro est implémenté ; les identifiants Max5c sont incompatibles";
    l.cli_bad_option = "option inconnue '{}'";
    l.cli_bad_channel = "le canal doit être 1–30";
    l.cli_bad_timeout = "le délai doit être 1–60 secondes";
    l.cli_need_value = "{} nécessite une valeur";
    l.cli_unknown_command = "commande inconnue '{}' ; exécutez ugreen --help";
    l.cli_bad_usage = "utilisation : {}";
    l.cli_bad_decode = "la capture n'est pas un flux de réponse propre et complet : {} trames, {} CRC rejetés, {} octets rejetés, {} trames inconnues, {} octets en attente";
    l.cli_no_tty = "La TUI exige des terminaux interactifs en entrée et sortie ; exécutez ugreen --help pour les commandes CLI.";
    l.cli_no_tui_feature =
        "La TUI est désactivée dans cette compilation CLI seule ; recompilez sans --no-default-features";
    l.cli_tui_dry =
        "tui ne se combine pas avec --dry-run ; utilisez --dry-run set CLÉ VALEUR pour des paquets hors ligne";
    l.cli_bad_profile = "le profil dépasse 16 Kio";
    l.cli_inflight_after_quit = "Une écriture en cours a peut-être abouti. Actualisez explicitement l'état de l'appareil avant toute autre modification.";
    l.set_bad_anc = "valeur ANC invalide";
    l.set_bad_eq = "valeur EQ invalide";
    l.set_bad_onoff = "on ou off attendu";
    l.set_bad_prompts = "voice ou beeps attendu";
    l.set_bad_button = "none, next ou previous attendu";
    l.set_unknown = "réglage inconnu '{}'\n{}";
    l.profile_big = "le profil dépasse 16 Kio";
    l.profile_line = "la ligne {} du profil doit être clé=valeur";
    l.profile_dup = "clé de profil en double '{}'";
    l.profile_model = "le modèle du profil doit être studio-pro";
    l.profile_need_model = "le profil exige model=studio-pro";
    l.profile_empty = "le profil n'a aucun réglage";
    l.req_timeout = "aucune réponse valide avant l'échéance";
    l.req_closed = "connexion Bluetooth fermée";
    l.req_rejected = "les écouteurs ont rejeté l'instruction 0x{:02X}";
    l.info_short = "charge utile d'info appareil trop courte : {} octets (8 requis au minimum)";
    l.fw_short = "la réponse firmware n'a pas d'octets de version";
    l.set_ackfail = "le réglage a peut-être été envoyé mais la confirmation a échoué : {} ; interrogez l'état avant de réessayer";
    l.set_rbfail = "écriture confirmée mais le relevé a échoué : {}";
    l.set_rbfail_noack = "écriture envoyée mais le relevé a échoué : {}";
    l.set_mismatch =
        "écriture confirmée mais le relevé de {} ne correspond pas à {} ; interrogez l'état";
    l.set_mismatch_noack =
        "écriture envoyée mais le relevé de {} ne correspond pas à {} ; interrogez l'état";
    l
}
