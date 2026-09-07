# Produção da vitrine — 7 de setembro de 2026

A página `index.html` usa capturas do aplicativo real no Linux, gravadas em uma instância exclusiva do RamDog dentro de uma bancada X11 de 1600 × 1000. Os clientes das janelas foram recortados para as imagens. A demonstração mostra a revisão de desenvolvimento 0.9.1; o botão de download acompanha a release estável publicada. Não houve alteração de serviços, PWM, monitores ou aplicativos da sessão humana.

## Evidência e medidas

- Vídeo H.264/AAC: 1920 × 1080 para o arquivo original e 1280 × 720 para a página; 30 fps, duração de 85,1 s. Validado por decodificação completa, folhas de contato de cada cena e do vídeo final.
- Narração sintetizada localmente no OmniVoice com voz autorizada do proprietário. Dez falas, 74,4 s de voz; WAVs 48 kHz estéreo normalizados para −16 LUFS, sem clipping. Conteúdo conferido por transcrição local. Legendas em português acompanham o vídeo.
- Binário Linux x86_64 da revisão demonstrada, compilado localmente com `cargo build --locked --release`: **7.401.600 bytes (7,06 MiB)**. É tamanho em disco, não uso de RAM nem promessa para outros builds.
- O exemplo de encerramento usa um processo `ramdog-demo` e dois filhos `ramdog-worker`, criados exclusivamente para a gravação. O pai foi encerrado pelo RamDog; os dois filhos protegidos permaneceram ativos. Todos foram removidos ao encerrar a bancada.
- As métricas que aparecem nas telas são leituras da máquina durante trabalho concorrente. Não são benchmarks, ganhos de desempenho ou estimativas de memória economizada.
- Partida e Desperdício mostram o inventário acessível do sistema. O aviso de barramento de usuário ausente corresponde ao isolamento da bancada. Telas mostra seu aviso real: o backend Linux precisa de Hyprland e não funciona no X11 usado nesta gravação.
- Página conferida em 1440 px e 390 px: imagens carregadas, âncoras válidas, ausência de overflow horizontal externo e reprodução de vídeo com legendas. As imagens completas podem ser abertas a partir de cada seção.

## Fontes das afirmações

As funcionalidades e limites do RamDog foram confrontados com o código, o [README](../README.md), a [documentação Linux](../linux/README.md), o [changelog](../CHANGELOG.md) e a [licença MIT](../LICENSE). A comparação da página descreve o escopo documentado, sem concluir que uma função inexiste apenas porque não aparece num manual.

Fontes externas consultadas em 07/09/2026:

- [Microsoft — Task Manager](https://learn.microsoft.com/en-us/troubleshoot/windows-server/support-tools/support-tools-task-manager)
- [Microsoft — configuração de aplicativos de inicialização](https://support.microsoft.com/en-us/windows/experience/startup-boot/configure-startup-applications-in-windows)
- [htop — manual](https://github.com/htop-dev/htop/blob/main/htop.1.in)
- [htop — projeto](https://htop.dev/)

Os números de CPU/RAM e a disponibilidade de sensores variam entre máquinas e instantes. `—` significa indisponível; o lock protege contra as ações do próprio RamDog.
