# Pasos para la demo

1. (servidor remoto 1) levantamos el servidor en otra carpeta, y creamos repo remoto con git init remoto1.git —bare
2. (cliente 1) En una carpeta creamos repo con init
3. (cliente 1) Pegamos un proyecto que ya tengamos preparado con archivos y carpetas, con algunos .gitignore
4. (cliente 1) verificamos que en status no figuran los archivos ignorados
5. (cliente 1) Hacemos add de todos los archivos
6. (cliente 1) Hacemos el commit inicial
7. (cliente 1) Hacemos una annotated tag para HEAD: add ann_tag_repo1 mensaje
8. (cliente 1) Listamos tags, y visualizamos el tag haciendo cat-file del commit que está en .git/refs/tags/nombre_de_tag que tiene el contenido del tagger y demas info
9. (cliente 1) Hacemos branch rama1 y checkout rama1
10. (cliente 1) Desde rama1 hacemos modificaciones y un commit
11. (cliente 1) Checkout a master, modificaciones, y commit
12. (cliente 1) seteamos en el cliente este servidor como remoto: add origin git://127.0.0.1:9418/remoto1.git
13. (cliente 1) hacemos un push de lo que tenemos: origin master
14. (cliente 2) En otra carpeta hacemos un clone usando el link del repo remoto git://127.0.0.1:9418/remoto1.git
15. (cliente 2) Hacemos checkout a rama1, modificamos y hacemos commit
16. (cliente 2) hacemos git branch para visualizar ambas ramas
17. (cliente 2) Hacemos push
18. (cliente 1) Hacemos checkout rama1
19. (cliente 1) Hacemos pull con origin
20. (cliente 1) Hacemos checkout a master y commiteamos algun cambio (tiene que dar conflicto con rama1)
21. (cliente 1) Mergeamos a rama1
22. CAMBIAR AL CLIENTE 2
23. (cliente 2) checkout a master
24. (cliente 2) Lo mismo en cliente 2, esta vez con rebase (ver que se aplano la historia)
    * Commitear en master
    * Checkout a rama1
    * Rebase con master
