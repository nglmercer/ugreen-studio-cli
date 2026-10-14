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
    l.cli_models = "studio-pro : protocole basé sur le code de référence UGREEN Studio Pro HP206 et sa capture\nStatut du test matériel : VÉRIFIÉ sur le firmware retail 0.2.5 (Linux, canal 1 ; voir docs/verification.md)\nHiTune Max5c : NON PRISE EN CHARGE (identifiants de commande conflictuels)\nAutres modèles/firmwares UGREEN : NON VÉRIFIÉS";
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
    // Fragments d'aide générée ; l'écran
    // est rendu à partir du registre de commandes.
    l.help_subtitle = "CLI Bluetooth non officielle pour UGREEN Studio Pro";
    l.help_usage = "UTILISATION";
    l.help_offline = "COMMANDES HORS LIGNE";
    l.help_bluetooth = "COMMANDES BLUETOOTH (appairez d'abord dans les paramètres système)";
    l.help_capture_gated = "REQUÊTES À PORTE (refusées avant la connexion pour le moment)";
    l.help_options = "OPTIONS (avant la COMMANDE)";
    l.help_examples = "EXEMPLES";
    l.help_footer = "Protocole Studio Pro HP206 vérifié sur le firmware retail 0.2.5 (un appareil) ;\nHiTune Max5c utilise des identifiants de commande conflictuels. Aucune action de\nfirmware, réinitialisation, écriture brute ou recherche de casque n'est fournie. Les profils peuvent s'appliquer partiellement en cas d'échec.";
    l.opt_address = "Adresse Bluetooth explicite de la cible";
    l.opt_model = "Requis pour les requêtes matérielles ; jamais détecté automatiquement";
    l.opt_channel = "Canal RFCOMM, 1–30 (par défaut 1)";
    l.opt_timeout = "Délai par opération en secondes, 1–60";
    l.opt_lang =
        "Langue de l'interface : en, es, pt, de, fr, it, nl, ru, zh, ja, ko (par défaut en)";
    l.opt_autoconnect = "Se connecter au dernier appareil au démarrage (par défaut oui)";
    l.opt_no_autoconnect = "Désactiver la connexion automatique au démarrage";
    l.opt_json = "JSON versionné pour les commandes de lecture compatibles";
    l.opt_dry_run = "Imprimer les paquets de réglages sans accès Bluetooth";
    l.opt_help = "Afficher l'aide";
    l.opt_version = "Afficher la version";
    l.cmd_help = "Afficher cette aide";
    l.cmd_tui = "Ouvrir l'interface de terminal interactive optionnelle";
    l.cmd_models = "Afficher la compatibilité du protocole et les limites";
    l.cmd_commands = "Lister les réglages pris en charge";
    l.cmd_decode = "Valider/décoder hors ligne des octets de réponse capturés";
    l.cmd_analyze = "Interpréter hors ligne les résultats statiques de l'APK";
    l.cmd_candidate = "Imprimer un candidat de paquet statique ; n'est jamais envoyé";
    l.cmd_profile = "Gérer les profils de réglages";
    l.cmd_profile_example = "Imprimer un exemple de profil de réglages";
    l.cmd_profile_export = "Lire les réglages et imprimer un profil réutilisable";
    l.cmd_profile_apply = "Valider le profil, appliquer et vérifier chaque réglage";
    l.cmd_discover = "Lister les appareils appairés en cache ; aucune analyse radio";
    l.cmd_capture = "Enregistrer les octets TX/RX bruts d'une session en hexadécimal de test";
    l.cmd_status = "Lire la batterie, le firmware et les réglages";
    l.cmd_set = "Modifier un réglage puis vérifier par lecture";
    l.cmd_query = "Requête à porte ; non vérifiée sur ce firmware";
    l.cmd_control = "Contrôle à porte ; --dry-run imprime les octets";
    l.cmd_find = "Ton de recherche à porte";
    l.headset_connections = ["Connexions du casque", "Ces appareils sont reliés au casque, distincts des appareils associés à ce PC.", "La requête HP206 (0x0D) nécessite des captures de l’application officielle pour ce firmware. Aucune requête envoyée ; nombre et identités inconnus.", "Déconnexion, reconnexion et changement des appareils distants non pris en charge. Les actions Bluetooth hôte concernent uniquement ce PC.", "r : vérifier · d : connexion double · Échap : retour"];
    l.page_labels = [
        "Appareils",
        "Audio",
        "Réglages du casque",
        "Multipoint",
        "Fonctions expérimentales",
        "Diagnostic",
    ];
    l.page_hint = "↑/↓ Entrée : ouvrir · r : actualiser · Échap : retour";
    l.host_commands = [
        "Observer le transport audio du système",
        "Lister les sorties audio locales",
        "Afficher les preuves des fonctions",
        "Lire le diagnostic local",
        "Connecter la liaison Bluetooth locale",
        "Déconnecter la liaison Bluetooth locale",
    ];
    l.cmd_audio_devices = "Lister les sorties audio Windows";
    l.features_title = " Fonctionnalités ";
    l.features_search = "Rechercher : ";
    l.features_all = "Tout";
    l.feature_categories = [
        "Audio",
        "Connexion",
        "Environnement",
        "Retour",
        "Informations de l'appareil",
        "Multipoint",
        "Codecs",
        "Audio avancé",
        "Contrôles",
        "Partage audio",
        "Recherche",
        "Gestion de l'appareil",
        "Diagnostic",
    ];
    l.feature_names = [
        "ANC",
        "Égaliseur",
        "Mode jeu",
        "Audio spatial",
        "Double connexion",
        "Réduction du vent",
        "Annonces vocales",
        "Action volume +",
        "Action volume -",
        "Appareils multipoint",
        "Codec audio",
        "ID du casque",
        "Couleur du casque",
        "MAC du boîtier",
        "Contrôle LDAC",
        "LDAC pour un appareil",
        "Décodage haute qualité",
        "Commande de volume",
        "Canal mono/stéréo",
        "Mappage des boutons",
        "Actions IA",
        "Contrôle IA",
        "Désactiver le partage",
        "Partager avec un appareil",
        "Partager avec un appareil choisi",
        "Trouver l'écouteur gauche",
        "Trouver l'écouteur droit",
        "Réinitialisation d'usine",
        "Décodeur de capture",
        "Analyse de capture",
        "Aperçu de paquet",
    ];
    l.feature_descriptions = [
        "Profondeur de réduction active du bruit. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Préréglage de l'égaliseur. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Interrupteur de faible latence du mode jeu. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Interrupteur audio spatial. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Interrupteur de connexion à deux appareils. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Interrupteur de réduction du bruit de vent. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Style des annonces vocales. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Action volume plus maintenue. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Action volume moins maintenue. Réglage vérifié ; modifiez-le dans les Réglages.",
        "Lire la liste des appareils multipoint connectés. Le protocole d'appairage requiert encore une vérification matérielle.",
        "Lire le codec audio Bluetooth actif. Le format de requête requiert encore une vérification matérielle.",
        "Lire l'ID matériel du casque. Le format de requête requiert encore une vérification matérielle.",
        "Lire la couleur du casque. Le format de requête requiert encore une vérification matérielle.",
        "Lire l'adresse MAC du boîtier de charge. Le format de requête requiert encore une vérification matérielle.",
        "Basculer le codage LDAC sur la connexion actuelle. Le format de contrôle requiert encore une vérification matérielle.",
        "Basculer le codage LDAC pour un appareil appairé. Le format de contrôle requiert encore une vérification matérielle.",
        "Basculer le décodage audio haute qualité. Le format de contrôle requiert encore une vérification matérielle.",
        "Envoyer un octet de volume absolu. La plage matérielle n'est pas vérifiée.",
        "Commuter le canal audio entre mono et stéréo. Le format de contrôle requiert encore une vérification matérielle.",
        "Remapper les huit actions de maintien. Le format de contrôle requiert encore une vérification matérielle.",
        "Remapper les quatre actions déclenchées par l'IA. Le format de contrôle requiert encore une vérification matérielle.",
        "Basculer le contrôle audio par IA. Le format de contrôle requiert encore une vérification matérielle.",
        "Désactiver le partage audio. Le format de contrôle requiert encore une vérification matérielle.",
        "Partager l'audio avec l'unique appareil connecté. Le format de contrôle requiert encore une vérification matérielle.",
        "Partager l'audio avec un appareil choisi. Le format de contrôle requiert encore une vérification matérielle.",
        "Jouer un ton de localisation sur l'écouteur gauche via l'hôte. Le protocole de recherche requiert encore une vérification matérielle.",
        "Jouer un ton de localisation sur l'écouteur droit via l'hôte. Le protocole de recherche requiert encore une vérification matérielle.",
        "Supprimer l'appairage et restaurer les valeurs d'usine. Destructif ; le format de contrôle requiert encore une vérification matérielle.",
        "Décoder des octets de réponse capturés hors ligne ; sans accès Bluetooth.",
        "Interpréter un flux de capture complet hors ligne ; sans accès Bluetooth.",
        "Afficher le paquet statique qu'un contrôle enverrait ; il n'est jamais transmis.",
    ];
    l.feature_available = "Disponible";
    l.feature_pending = "Vérification en attente";
    l.feature_pending_reason =
        "Le format filaire requiert encore une vérification matérielle avec l'application officielle.";
    l.feature_live_disabled =
        "L'exécution en direct est désactivée jusqu'à preuve vérifiée sur l'appareil.";
    l.feature_offline_workflow = "Analyse hors ligne uniquement ; rien n'est envoyé au casque.";
    l.feature_offline_hint = "Exécuter : ugreen {}";
    l.feature_kind_setting = "Réglage vérifié";
    l.feature_kind_query = "Requête d'appareil";
    l.feature_kind_control = "Contrôle d'appareil";
    l.feature_kind_offline = "Analyse hors ligne";
    l.feature_category_label = "Catégorie : {}";
    l.feature_current = "Actuel : {}";
    l.feature_edit_in_settings = "Modifier dans Réglages (s)";
    l.feature_run_query = "Entrée : exécuter la requête";
    l.feature_configure = "Entrée : configurer";
    l.feature_back = "Échap : retour";
    l.feature_form_title = " Configurer {} ";
    l.feature_err_required = "Tous les champs doivent être remplis avant la vérification";
    l.feature_field_unset = "non défini";
    l.feature_form_hint = "Haut Bas champ / Gauche Droite choix / Entrée vérifier / Échap retour";
    l.feature_review_title = " Vérifier {} ";
    l.review_write_title = " Vérifier l'écriture ";
    l.feature_review_target = "Cible : {}";
    l.feature_review_previous = "Actuel : {}";
    l.feature_review_proposed = "Proposé : {}";
    l.feature_review_values = "Valeurs : {}";
    l.feature_review_readback = "Relecture : {}";
    l.feature_readback_verified = "vérifié par accusé de réception et relecture correspondante";
    l.feature_readback_noack =
        "vérifié par relecture correspondante (l'audio spatial n'envoie pas d'accusé)";
    l.feature_readback_unconfirmed =
        "non confirmé de façon indépendante ; actualisez après application";
    l.feature_readback_query = "requête en lecture seule";
    l.feature_confirm_apply = "Entrée : appliquer    Échap : annuler";
    l.feature_confirm_reset =
        "Destructif : supprime l'appairage de l'hôte et restaure les valeurs d'usine.";
    l.features_footer = [
        [
            ("Haut", "déplacer"),
            ("Bas", "déplacer"),
            ("Tab", "catégorie"),
            ("Entrée", "ouvrir"),
            ("Échap", "retour"),
            ("", ""),
        ],
        [
            ("lettres", "rechercher"),
            ("Retour", "effacer"),
            ("Ctrl+U", "effacer"),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
        [
            ("Esc", "fermer"),
            ("?", "raccourcis"),
            ("", ""),
            ("", ""),
            ("", ""),
            ("", ""),
        ],
    ];
    l.status_feature_running = "Exécution de l'opération de fonctionnalités...";
    l.status_edit_in_settings = "{} est un réglage vérifié ; modifiez-le dans les Réglages (s).";
    l.codec_unverified = "requête non vérifiée";
    l.arg_enabled = "Activé";
    l.arg_side = "Côté";
    l.arg_duration = "Durée (s)";
    l.arg_target = "Cible";
    l.arg_value = "Valeur (0-255)";
    l.arg_mode = "Mode";
    l.arg_peer = "Appairé";
    l.arg_actions = "Actions";
    l.arg_address = "MAC de l'appairé";
    l.arg_hint_toggle = "on ou off";
    l.arg_hint_side = "gauche ou droite";
    l.arg_hint_seconds = "1..30 secondes";
    l.arg_hint_byte = "0..255";
    l.arg_hint_mac = "six octets hexadécimaux séparés par deux-points";
    l.arg_hint_peer = "1 ou 2";
    l.arg_hint_actions = "{} valeurs d'action séparées par des virgules";
    l.tui_no_tty =
        "L'interface exige des terminaux interactifs stdin et stdout ; exécutez ugreen --help pour les commandes de la CLI.";
    l.tui_bad_channel =
        "Le canal de l'interface doit être 1-30 et le délai supérieur à zéro et au plus 60 secondes.";
    l.status_worker_busy =
        "Une opération est déjà en cours ; attendez ou appuyez sur Échap pour annuler.";
    l.status_worker_sequence = "Séquence de requêtes épuisée ; redémarrez l'interface.";
    l.status_worker_stopped = "Le worker Bluetooth est arrêté ; redémarrez l'interface.";
    l.status_worker_queue = "La file du worker est pleine ; attendez avant de réessayer.";
    l.status_reset_unavailable = "cible de réinitialisation indisponible";
    l.status_reset_done =
        "Réinitialisation confirmée ; jumelage hôte et appareil enregistré supprimés.";
    l.status_inflight_write =
        "Une écriture en cours a peut-être abouti ; actualisez après le redémarrage.";
    l
}
