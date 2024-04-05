# URL SHORTENER

Quick and dirty URL shortener and server.


## Build the application
Run: `cargo build -r`

## Running the application
 - Using cargo
  
   `cargo run serve`
 - Running the binary

   `./url_shortener serve`

## Generate an API token
 - Generating a token for the default domain:

    `./url_shortener token`  or `cargo run token`
-  Generating a token for a specific domain:
   
   `./url_shortener token --domain https://foo.bar` or `cargo run token --domain https//foo.bar`

## Using the REST API

- generate a new short url:

  method: post
  paylaod: 

```json
  {
    "raw": "the URL to shorten"
    "expires": "timestamp in milliseconds or zero"
  }
```
  success response:

```json
  {
	"success": true,
	"data": {
		"id": "01htphvyh07qmbgpdnj9cyagbq",
		"url": "http://127.0.0.1:3000/RzswBz",
		"expires": 0
	},
	"error": null
}
```
example of an unsccessful request:

```json
{
	"success": false,
	"data": null,
	"error": {
		"code": "wrong_timestamp",
		"message": "The timestamp must be in the future and in milliseconds"
	}
}
```
## Prune expired URLs from the database


`./url_shortener prune` or `cargo run prune`