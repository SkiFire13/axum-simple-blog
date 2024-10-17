# A simple blog

This is a simple web application build with Rust and SQLite.

## Running with Docker

In order to run the web application with Docker first build the container with the following command:

- First, clone this repository
    ```sh
    git clone https://github.com/SkiFire13/simple-blog && cd simple-blog
    ```

- Then, build the Docker container

    ```sh
    docker build -t blog .
    ```

- Finally, run the application. Remember to replace `$PORT` with the port you want to externally expose the website on.
  This will also create a Docker volume  `blog-data` to persist the blog data. You can change it to another name if you
  like, or use a file system path to mount a directory instead of a Docker volume.

   ```sh
   docker run -p $PORT:80 -v blog-data:/app/data
   ```
