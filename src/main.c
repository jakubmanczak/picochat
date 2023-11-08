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
#define MAXCLIENTS 32
#define MSGBUFFER 256
#define NAMEBUFFER 32
#define LOGBUFFER (MSGBUFFER + NAMEBUFFER + 32)

typedef struct client{
  int connfd;
  // char *name;
} client;

int sockfd;
char *welcomemsg = "reached a picochat server.\n";
fd_set sockets, readySockets;

void setupServerNetworking(){
  struct addrinfo hints, *res; int yes = -1;
  
  memset(&hints, 0, sizeof hints);
  hints.ai_family = AF_UNSPEC; hints.ai_socktype = SOCK_STREAM; hints.ai_flags = AI_PASSIVE;
  getaddrinfo(NULL, PORT, &hints, &res);
  sockfd = socket(res->ai_family, res->ai_socktype, res->ai_protocol);
  bind(sockfd, res->ai_addr, res->ai_addrlen);
  setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof yes);
  listen(sockfd, BACKLOG);
}

int main(void){
  setupServerNetworking();
  FD_ZERO(&sockets);
  FD_SET(sockfd, &sockets);

  char *buffer = malloc(MSGBUFFER); int rv;
  printf("picochat server up.\n");
  while(1){
    readySockets = sockets; // select is destructive
    select(FD_SETSIZE, &readySockets, NULL, NULL, NULL);

    for(int i = 0; i < FD_SETSIZE; i++){
      if(FD_ISSET(i, &readySockets)){
        if(i == sockfd){
          client cl; cl.connfd = 0;

          struct sockaddr_storage connstore;
          socklen_t addrsize;
          addrsize = sizeof connstore;
          cl.connfd = accept(sockfd, (struct sockaddr *)&connstore, &addrsize);
          FD_SET(cl.connfd, &sockets);
          printf("conn fd:%d accepted.\n", cl.connfd);
        }else{
          rv = recv(i, buffer, MSGBUFFER, 0);
          if(rv == 0){
            FD_CLR(i, &sockets);
            printf("conn fd:%d dropped.\n", i);
            close(i);
          }else{
            printf("%d: %s", i, buffer);
          }
          memset(buffer, 0, MSGBUFFER);
        }
      }
    }
  }
  return 0;
}