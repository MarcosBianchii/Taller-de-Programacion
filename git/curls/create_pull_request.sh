curl --location '127.0.0.1:8080/repos/remoto1/pulls' \
--header 'Content-Type: application/json' \
--data '{
    "base": "branch1",
    "head": "master",
    "title": "Titulo de Pull Request",
    "body": "Cuerpo de Pull Request",
    "owner": "KrabbyPatty"
}'