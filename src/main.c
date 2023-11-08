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

char *welcomemsg = "You've reached a picochat server.\n";
int sockfd; int clients = 0;

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

  while(1){
    client cl; cl.connfd = 0;

    struct sockaddr_storage connstore;
    socklen_t addrsize;

    addrsize = sizeof connstore;
    cl.connfd = accept(sockfd, (struct sockaddr *)&connstore, &addrsize);
    printf("conn fd:%d accepted\n", cl.connfd);
    if(!fork()){
      char *buffer = malloc(MSGBUFFER); int rv = 0xbeef;
      close(sockfd); clients++;
      while(1){
        rv = recv(cl.connfd, buffer, MSGBUFFER, 0);
        if(rv > 0){
          printf("%d: %s", cl.connfd, buffer);
          memset(buffer, 0, MSGBUFFER);
        }else if(rv == 0){
          printf("conn fd:%d dropped\n", cl.connfd);
          close(cl.connfd); free(buffer);
          exit(0);
        }else{
          // error occured
          printf("recv() error!\n");
          exit(1);
        }
      }
    }
  }
  return 0;
}