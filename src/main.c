#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>
#include <arpa/inet.h>

#define PORT "2004"
#define BACKLOG 8

char *welcomemsg = "You're connected to picochat.\n";

int main(void){
  int sockfd; struct addrinfo hints, *res; int yes = 1;

  memset(&hints, 0, sizeof hints);
  hints.ai_family = AF_UNSPEC; hints.ai_socktype = SOCK_STREAM; hints.ai_flags = AI_PASSIVE;
  getaddrinfo(NULL, PORT, &hints, &res);
  sockfd = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
  bind(sockfd, res->ai_addr, res->ai_addrlen);
  setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof yes);
  listen(sockfd, BACKLOG);

  int connfd = 0;
  int buffersize = 256;
  char *buffer = malloc(buffersize * sizeof(char));

  while(1){
    if(!connfd){
      struct sockaddr_storage connstore;
      socklen_t addrsize;

      addrsize = sizeof connstore;
      connfd = accept(sockfd, (struct sockaddr *)&connstore, &addrsize);
      send(connfd, welcomemsg, strlen(welcomemsg), 0);
      printf("Connection (%d) accepted.\n", connfd);
    }else{
      memset(buffer, 0, buffersize);
      int rv = recv(connfd, buffer, buffersize, 0);
      if(rv == 0){
        close(connfd); printf("Connection (%d) closed.\n", connfd); connfd = 0;
      }else{
        printf("%d says: %s", connfd, buffer);
        send(connfd, "received message:\n", 19, 0);
        send(connfd, buffer, strlen(buffer), 0);
        send(connfd, "\n", 2, 0);
      }
    }
  }

  return 0;
}