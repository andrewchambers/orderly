package main

import (
	"fmt"
	"log"
	"net/http"
)

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		log.Print("Got a request: %#v", r)
		fmt.Fprintf(w, "Welcome to my website!")
	})

	log.Print("server listening for requests on port 8000")
	log.Fatal(http.ListenAndServe("127.0.0.1:8000", nil))
}
