# Taller de programación I - Cátedra DEYMONNAZ​

### GRUPO: LOS KRABBY PATTY 

### Integrantes

- Francisco Insua  
- Marcos Bianchi
- Felipe Sagasti
- Rebeca Davila


## Introduccion

La resolución del trabajo práctico implicó el desarrollo de una arquitectura cliente-servidor similar a la herramienta
de control de versiones Git, que soportara los comandos básicos de esta herramienta para la gestión de proyectos 
locales, tales como add, commit, status, branch, checkout, etc. así como también la interacción con repositorios remotos 
alojados en un servidor, respetando el protocolo Git  Transport. El proyecto debía contar además con una interfaz gráfica
simple que permitiera tanto utilizar los comandos anteriormente mencionados, así como también visualizar el historial 
de commits de las ramas.


## Investigación

Para la resolución del proyecto, se consultó diferentes fuentes. Principalmente la página oficial de git “https://git-scm.com”,
y en particular el capítulo 10 del libro “Git internals”, donde se desarrollan los conceptos de git objects, Plumbing and 
Porcelain commands,  Packfiles, entre otros, que fueron fundamentales para entender el funcionamiento interno de Git y poder 
así idear una solución propia a partir de esta. 

 
## Diseño de solución

Por la naturaleza del proyecto, se comenzó desarrollando el trabajo práctico a partir de dos módulos: client y server.  
El modulo client contenía las estructuras, módulos y librerías necesarias para el funcionamiento de un cliente git que permita 
realizar operaciones de gestión de versiones locales mediante una interfaz gráfica, así como también interaccionar con un 
servidor mediante el protocolo Git Protocol. El módulo server contenía los módulos y librerías necesarias para atacar múltiples
solicitudes en simultáneo, procesar paquetes de datos enviados, y enviar paquetes de datos solicitados. Al avanzar con la 
implementación de estos módulos resultó evidente que para realizar la interacción entre cliente y servidor o levantar/grabar objetos
en la base de datos, ambos utilizaban métodos idénticos, por lo que se optó por crear un tercer módulo llamado utils en el que 
se volcó la parte compartida por cliente y servidor de la obtención y grabado de objetos desde y hacia la base de datos, y el 
formateo y procesamiento de la información a enviar/recibir entre cliente y servidor (pack file).


![Alt text](<imagenes/diagrama_general.png>)

A continuación se realiza un análisis de cada uno de los módulos.

### cliente

La implementación del cliente consta de ocho módulos principales. Los siguientes son:

* **commands**: contiene la funcionalidad básica de un cliente git. Comandos como init, branch, pull, push, etc.

* **index**: Se encarga del trackeo de archivos dentro del sistema, determina cuales son los objetos que conformarán el próximo
commit

* **plumbing**: contiene la implementación interna de muchos de los comandos definidos en commands, junto con comandos de plumbing
accesibles al usuario, pero poco utilizados por ser parte de la implementación de comandos "porcelain" más amigables para los usuarios.

* **protocol**: Implementa parte de la funcionalidad usada por el cliente para comunicarse con el servidor git.

* **interfaz**: el módulo ui (user interface) contiene las pantallas para la interacción del usuario con la herramienta, junto con 
la lógica de llamado a los comandos implementados en command o con el input ingresado por el usuario. 

* **diff**: Contiene funciones para calcular diferencias entre commits, trees y archivos de texto.

* **config_file**: Su propósito es manejar el archivo de config del repositorio. Relaciona ramas y remotos mediante este archivo de texto.

![Alt text](<imagenes/client_diagram.png>)

## servidor

La implementación del servidor consta de tres módulos principales:

* **server**: contiene la funcionalidad para que sea posible comunicarse con un cliente git que realiza solicitudes mediante el protocolo
Git Transport, tales como receive-pack o upload-pack, las cuales se utilizan para enviar información al servidor y solicitar información 
de éste, respectivamente. 

* **ThreadPool**: contiene la lógica que permite al servidor manejar de forma simultánea más de una solicitud de clientes a la vez, utilizando 
una cantidad fija de workers activos que esperan a que se les asigne un clousure para ejecutarlo.

* **worker**: contiene la lógica para inicializar un worker (thread) que espera recibir un mensaje de su threadpool para ejecutar un trabajo.

![Alt text](<imagenes/server_diagram.png>)

## utils

La implementación de utils consta de cuatro mmódulos:

* **pack**: contiene la lógica que permite leer un pack-file, procesarlo, y registrar el contenido, los objetos, en la base de datos. A su 
vez permite crear un pack-file a partir de objetos de la base de datos.

* **PackEntry**: utilizado por módulo pack para generar cada una de las entradas del pack-file a partir de objetos, o para registrar las 
entradas del pack en la base de datos.

* **object**: contiene funcionalidad que permite obtener la información de un objeto alojado en la base de datos a partir de su hash.

* **plumbing**: contiene la funcionalidad que permite registrar un objeto en la base de datos, visualizar los componentes de un tree object, 
obtener el root tree de un commit, o los padres de un commit.

![Alt text](<imagenes/utils_diagram.png>)

## Diagramas de secuencia interacción cliente/servidor
Diagrama de función fetch

![Alt text](<imagenes/fetch_diagram.png>)

Diagrama de función push

![Alt text](<imagenes/push_diagram.png>)

## Conclusiones

El desarrollo del trabajo práctico representó un desafío para los integrantes del grupo, tanto en las complejidades del trabajo en equipo, 
en el planeamiento y diseño de un proyecto de mediana envergadura, en la adaptación de nuestro proyecto para que permita la interoperabilidad 
con clientes y servidores de Git (lo que implicó seguir al pie de la letra documentación técnica de protocolos provista por los desarrolladores 
de Git) y en el manejo de los tiempos para llegar a las entregas. 
A medida que se avanzó en la resolución, se pudo encontrar el ritmo para poder adelantar de forma más constante, y dando pasos más firmes.
A pesar de las dificultades, el proyecto representó una oportunidad de aprendizaje sobre desarrollo en equipos de trabajo, y una oportunidad 
de aprender a fondo el funcionamiento de una de las herramientas de software más utilizadas en la industria. 