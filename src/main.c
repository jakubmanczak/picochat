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

  while(1){
    struct sockaddr_storage connstore;
    socklen_t addrsize;
    int connfd;

    addrsize = sizeof connstore;
    connfd = accept(sockfd, (struct sockaddr *)&connstore, &addrsize);

    char *msg = "testmsg\n";

    send(connfd, welcomemsg, strlen(welcomemsg), 0);
    
    close(connfd);
  }

  return 0;
}