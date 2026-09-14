# Produção da vitrine — 7 de setembro de 2026

A página `index.html` usa capturas do aplicativo real no Linux, gravadas em uma instância exclusiva do RamDog dentro de uma bancada X11 de 1600 × 1000. Os clientes das janelas foram recortados para as imagens. A demonstração mostra a revisão de desenvolvimento 0.9.1; o botão de download acompanha a release estável publicada. Não houve alteração de serviços, PWM, monitores ou aplicativos da sessão humana.

## Evidência e medidas

- Vídeo H.264/AAC: 1920 × 1080 para o arquivo original e 1280 × 720 para a página; 30 fps, duração de **82,966667 s** (83 s na página), após a recomposição com voz corrigida. São 2489 quadros, com duração de áudio e vídeo conferida por ffprobe e decodificação completa. A folha de contato dos dez segmentos foi inspecionada.
- Narração sintetizada localmente no OmniVoice com voz autorizada do proprietário. A recomposição usa os dez WAVs corrigidos selecionados pelo coordenador, em 48 kHz estéreo e aproximadamente −16 LUFS. Nenhuma fala nova foi gerada nesta recomposição, nem houve alteração de velocidade, nova normalização, ganho, fade ou corte dos WAVs. O PCM foi colocado integralmente na linha de tempo; o master recebeu uma codificação AAC e a versão web copia essa faixa. Há pelo menos 1 s depois de cada WAV. As 17 entradas das legendas foram recalculadas, preservando o texto aprovado.
- Binário Linux x86_64 da revisão demonstrada, compilado localmente com `cargo build --locked --release`: **7.401.600 bytes (7,06 MiB)**. É tamanho em disco, não uso de RAM nem promessa para outros builds.
- O exemplo de encerramento usa um processo `ramdog-demo` e dois filhos `ramdog-worker`, criados exclusivamente para a gravação. O pai foi encerrado pelo RamDog; os dois filhos protegidos permaneceram ativos. Todos foram removidos ao encerrar a bancada.
- As métricas que aparecem nas telas são leituras da máquina durante trabalho concorrente. Não são benchmarks, ganhos de desempenho ou estimativas de memória economizada.
- Partida e Desperdício mostram o inventário acessível do sistema. O aviso de barramento de usuário ausente corresponde ao isolamento da bancada. Telas mostra seu aviso real: o backend Linux precisa de Hyprland e não funciona no X11 usado nesta gravação.
- Página conferida em 1440 px e 390 px: imagens carregadas, âncoras válidas, ausência de overflow horizontal externo e reprodução de vídeo com legendas. As imagens completas podem ser abertas a partir de cada seção.

## Recomposição da voz e guia completo

Os vídeos anteriores, as legendas e os posters foram preservados em `~/Videos/ramdog-vitrine/source/voz-anterior-20260908/`, com inventário de hashes. O roteiro de recomposição é `compose-voz-corrigida.json`; o relatório e os resultados de QA ficam em `source/RECOMPOSICAO-VOZ-CORRIGIDA.json` e `out-voz-corrigida/checks/` dentro da mesma pasta. As imagens originais do promo foram mantidas e as cenas foram estendidas quando a nova locução exigiu. A página recebeu os arquivos recompostos junto do guia completo.

A página [guia.html](guia.html) organiza o tutorial em **24 capítulos, 91 trechos narrados e 141 IDs editoriais**, com busca por recurso, transcrição em português e legendas alinhadas aos segmentos da narração. O vídeo completo dura **1760,80 s (29min21s)**, em 25 fps, com versões 1080p e 720p e capítulos incorporados. O player usa a versão 720p; o download oferece 1080p.

O roteiro e o mapa de cobertura distinguem 47 blocos com ação principal demonstrada e 44 blocos de explicação documental ou imagem real de apoio. Isso cobre os itens catalogados sem afirmar execução nativa de todas as combinações de sistema, permissão e hardware. O tutorial inclui instalação em destino exclusivo, configuração própria, processos descartáveis, serviço criado para a aula e janelas de dois programas sob um Hyprland separado. As bancadas e a unidade exclusiva foram encerradas e removidas ao terminar. As preferências e os serviços de uso do proprietário foram preservados.

Os 91 WAVs foram preservados integralmente na linha de tempo PCM, com início a 250 ms do começo de cada bloco e pelo menos 800 ms após seu término. O master recebeu uma codificação AAC; a versão web copia essa faixa. A auditoria de fala incluiu revisão seletiva de 16 blocos, sem diferenças restantes de frase ou sentido nesse lote. O processo de produção e os arquivos de auditoria ficam em `~/Videos/ramdog-guia-completo/`.

A verificação final confirmou os 91 blocos PCM sem mudança de amostras, 24 capítulos incorporados e 552 legendas ordenadas dentro da duração. A versão web foi decodificada integralmente sem erros. O player foi testado em 1440 px e 375 px, com reprodução após saltos de capítulo e de trecho, busca funcional, legendas carregadas, zero erro JavaScript e ausência de overflow horizontal. As folhas de contato dos 24 capítulos foram inspecionadas.

O material do guia diferencia gravação real no Linux em desenvolvimento 0.9.1, interface Windows v0.9.0 executada no Wine e explicações documentais. A telemetria nativa do Windows não foi validada. Defender, controle PWM físico e macOS recebem explicações documentais identificadas; não são apresentados como operações executadas nesta produção.

## Fontes das afirmações

As funcionalidades e limites do RamDog foram confrontados com o código, o [README](../README.md), a [documentação Linux](../linux/README.md), o [changelog](../CHANGELOG.md) e a [licença MIT](../LICENSE). A comparação da página descreve o escopo documentado, sem concluir que uma função inexiste apenas porque não aparece num manual.

Fontes externas consultadas em 07/09/2026:

- [Microsoft — Task Manager](https://learn.microsoft.com/en-us/troubleshoot/windows-server/support-tools/support-tools-task-manager)
- [Microsoft — configuração de aplicativos de inicialização](https://support.microsoft.com/en-us/windows/experience/startup-boot/configure-startup-applications-in-windows)
- [htop — manual](https://github.com/htop-dev/htop/blob/main/htop.1.in)
- [htop — projeto](https://htop.dev/)

Os números de CPU/RAM e a disponibilidade de sensores variam entre máquinas e instantes. `—` significa indisponível; o lock protege contra as ações do próprio RamDog.
