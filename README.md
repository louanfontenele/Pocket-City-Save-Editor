# Pocket City Save Editor 2.0

Um aplicativo desktop em Rust e egui para editar saves locais de **Pocket City** e **Pocket City 2**. Funciona sem conexão com a internet. O seletor de idioma oferece português brasileiro, inglês, japonês, francês, italiano, alemão, espanhol da Espanha, tcheco, coreano, russo, chinês simplificado, espanhol da América Latina e ucraniano.

## Como usar

1. Feche o jogo antes de editar o save.
2. Selecione o jogo na barra lateral. A biblioteca procura arquivos nas pastas padrão do Windows. Use **Abrir arquivo**, arraste um save para a janela ou adicione outra pasta.
3. Edite os campos em **Visão geral**, **Coleções**, **Progressão** ou **Campos avançados**. Em Pocket City, selecione a versão automática ou manual da cidade.
4. Clique em **Salvar alterações** para substituir o arquivo e guardar o original. **Exportar cópia** grava em um novo caminho. Para voltar a uma versão anterior, use **Backups**.

O editor cria backups em `.pocket-city-editor-backups` ao lado do save. A biblioteca distingue saves principais e históricos de Pocket City 2. Ao reabrir uma cidade nesse jogo, o próprio jogo escolhe qual arquivo carregar. Confira se o arquivo editado é a versão que pretende abrir.

| Jogo | Arquivos |
| --- | --- |
| Pocket City | `_city_<ID>.save` e `_user.save` no diretório de dados do Electron. No Windows, o editor procura em `%APPDATA%\com.codebrewgames.pocketcity` e `%APPDATA%\Pocket City`. |
| Pocket City 2 | `city_<ID>.es3`, históricos `_auto_` e `_manual_`, `global_settings`, `survival_global_settings` e `os_settings` em `%USERPROFILE%\AppData\LocalLow\Codebrew Games Inc_\Pocket City 2\pocketcity2`. |

Em **Visão geral**, os dois jogos permitem editar o nível e a experiência da cidade. O editor mostra o XP necessário para o próximo nível e permite zerar o progresso do nível atual.

No Pocket City 2, **Coleções e desbloqueios** inclui afinidade com 16 perfis de NPCs, presentes pendentes, 130 construções com requisitos de progressão, recursos, veículos, inventário, marcos e melhorias de sobrevivência. Também é possível configurar estoques mínimos para venda ou conversão automática. Os níveis de afinidade vão de 0 a 999. **Construções bloqueadas** grava o desbloqueio por construção e mantém o nível e o modo atuais da cidade. Custos e regras de posicionamento continuam sendo avaliados pelo jogo.

- **Progressão no Pocket City** libera as três construções dos baús, edita as 33 políticas com seus pré-requisitos e mostra as 66 missões. Para missões ativas, **Marcar objetivo cumprido** deixa a recompensa disponível para resgate no jogo.
- **Progressão no Pocket City 2** controla o estado das 52 pesquisas, o bem-estar dos animais adotados, qual animal acompanha o prefeito e a espera registrada para eventos. Esses controles funcionam com as opções atuais de sandbox da cidade. [Regras de progressão dos dois jogos](docs/STEAM_PROGRESSION.md)
- **Progressão → Economia** edita os impostos e permite zerar a dívida nos dois jogos. No Pocket City, 100% representa a cobrança padrão. No Pocket City 2, o número é a própria alíquota, entre 0% e 35%.

A cidade passa a ser classificada como sandbox ao ativar qualquer uma das quatro opções de sandbox na visão geral. O editor mostra a classificação a partir dos valores pendentes. [Regras de sandbox e ciclo dos saves do Pocket City 2](docs/POCKET_CITY_2_SAVES.md)

Os metadados da cidade de Pocket City 1, os IDs estruturais e as dimensões da malha aparecem somente para leitura. As ações de progressão gravam os campos associados à ação escolhida.

## Build local

```powershell
cargo run --release
cargo test
cargo clippy --all-targets -- -D warnings
```

O executável Windows fica em `target/release/pocket-city-save-editor.exe`. Use um caminho de arquivo como argumento opcional para abrir um save diretamente.

Consulte [SAVE_FORMATS.md](docs/SAVE_FORMATS.md) para detalhes da investigação e da compatibilidade.

## Publicação

Uma tag no formato `vX.Y.Z` que corresponda à versão de `Cargo.toml` aciona o GitHub Actions. O workflow executa os testes, compila o executável Windows x64 e publica um ZIP e seu checksum SHA-256 na release.
