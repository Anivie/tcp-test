#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <netinet/tcp.h>
#include <netinet/ip.h>
#include <stdlib.h>
#include <arpa/inet.h>
#include <errno.h>

// TCP伪头部
struct pseudo_header {
    uint32_t source_address;
    uint32_t dest_address;
    uint8_t placeholder;
    uint8_t protocol;
    uint16_t tcp_length;
};

// 校验和计算函数
unsigned short csum(unsigned short *ptr,int nbytes) {
    register long sum;
    unsigned short oddbyte;
    register short answer;

    sum=0;
    while(nbytes>1) {
        sum+=*ptr++;
        nbytes-=2;
    }
    if(nbytes==1) {
        oddbyte=0;
        *(u_char*)&oddbyte=*(u_char*)ptr;
        sum+=oddbyte;
    }

    sum = (sum>>16)+(sum & 0xffff);
    sum = sum + (sum>>16);

    return (short)~sum;
}

int main () {
    // 创建原始套接字
    int s = socket (PF_INET, SOCK_RAW, IPPROTO_TCP);
    if(s == -1) {
        perror("Failed to create socket");
        exit(1);
    }

    // 数据包
    char datagram[4096] = {0}, source_ip[32], *data, *pseudogram;

    // IP头
    struct iphdr *iph = (struct iphdr *) datagram;

    // TCP头
    struct tcphdr *tcph = (struct tcphdr *) (datagram + sizeof (struct ip));

    // 数据部分
    data = datagram + sizeof(struct iphdr) + sizeof(struct tcphdr);
    strcpy(data , "");

    struct sockaddr_in sin;
    struct pseudo_header psh;

    // 填充IP头
    iph->ihl = 5;
    iph->version = 4;
    iph->tos = 0;
    iph->tot_len = sizeof (struct iphdr) + sizeof (struct tcphdr) + strlen(data);
    iph->id = htonl (54321);
    iph->frag_off = 0;
    iph->ttl = 255;
    iph->protocol = IPPROTO_TCP;
    iph->check = 0;
    iph->saddr = inet_addr ("192.168.1.2");
    iph->daddr = sin.sin_addr.s_addr;

    // 填充TCP头
    tcph->source = htons (1234);
    tcph->dest = htons (80);
    tcph->seq = 0;
    tcph->ack_seq = 0;
    tcph->doff = 5;
    tcph->syn = 1;
    tcph->window = htons (5840);
    tcph->check = 0;
    tcph->urg_ptr = 0;

    // 填充伪头部
    psh.source_address = inet_addr("192.168.1.2");
    psh.dest_address = sin.sin_addr.s_addr;
    psh.placeholder = 0;
    psh.protocol = IPPROTO_TCP;
    psh.tcp_length = htons(sizeof(struct tcphdr) + strlen(data) );

    int psize = sizeof(struct pseudo_header) + sizeof(struct tcphdr) + strlen(data);
    pseudogram = malloc(psize);

    memcpy(pseudogram , (char*) &psh , sizeof (struct pseudo_header));
    memcpy(pseudogram + sizeof(struct pseudo_header) , tcph , sizeof(struct tcphdr) + strlen(data));

    // 计算TCP校验和
    tcph->check = csum( (unsigned short*) pseudogram , psize);

    // 计算IP校验和
    iph->check = csum ((unsigned short *) datagram, iph->tot_len);

    // 填充sockaddr_in结构体
    sin.sin_family = AF_INET;
    sin.sin_port = htons(65534);
    sin.sin_addr.s_addr = inet_addr ("0.0.0.0");

    // 发送数据包
    if (sendto (s, datagram, iph->tot_len ,	0, (struct sockaddr *) &sin, sizeof (sin)) < 0) {
        perror("sendto failed");
    }// 数据包发送成功
    else {
        printf ("Packet Send. Length : %d \n" , iph->tot_len);
    }

    return 0;
}
