#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

// #define DEBUG
// #undef DEBUG

//File descriptors
int mfd = 0;
int cfd = 0;

//Send buffers
char cmd_r[8] = " READ ";
char cmd_w[8] = " WRITE ";
char cycle_send[18];
char addr_send[44];

//Recv buffers
char cmd_recv[8];
char cycle_recv[18];
char addr_recv[44];

extern "C" int Setup(char *rqst_to_memory, char *resp_to_cpu)
{

    if (mkfifo(rqst_to_memory, 0666) != 0)
    {
#ifdef DEBUG
        printf("Error creating FIFO at %s\n", rqst_to_memory);
#endif
        // return 1;
    }

#ifdef DEBUG
    printf("CPU::rqst_to_memory created\n");
#endif

    if (mkfifo(resp_to_cpu, 0666) != 0)
    {
#ifdef DEBUG
        printf("Error creating FIFO at %s\n", resp_to_cpu);
#endif
        // return 1;
    }

#ifdef DEBUG
    printf("CPU::resp_to_cpu created\n");
#endif

    mfd = open(rqst_to_memory, O_WRONLY | O_NONBLOCK);
    while (mfd == -1)
    {
        mfd = open(rqst_to_memory, O_WRONLY | O_NONBLOCK);
    }

#ifdef DEBUG
    printf("CPU::MFD OPENED\n");
#endif

    cfd = open(resp_to_cpu, O_RDONLY | O_NONBLOCK);
    if (cfd == -1)
    {
        printf("Error opening FIFO for tracing at %s\n", resp_to_cpu);
        return 1;
    }
#ifdef DEBUG
    printf("CPU::CFD OPENED\n");
#endif

    return 0;
}

extern "C" int SendRqst(char *str)
{
    /* Request Format is:
	 *
	 * address cmd issued_cycle
	 * 
	 * Your request trace looks like:
	 *
	 * 0000000083000000 READ 100
	 * 0000000082000000 WRITE 160
	 */
    // printf("Sending %s\n",str);
    int i = write(mfd, str, 41);
#ifdef DEBUG
    if (i < 0)
    {
        printf("Send failed: %s\n", strerror(errno));
    }
#endif
    return i;
}

extern "C" int RecvResp()
{
    /* Response Format is:
	 * 
	 * address returned_cycle
	 *
	 * Your response trace looks like:
	 *
	 * 0000000083000000 100
	 * 0000000082000000 160
  */
    return read(cfd, addr_recv, 35);
}

extern "C" char *RecvRespString()
{
    strcat(addr_recv, cycle_recv);
    // printf("Recieving %s\n",addr_recv);
    return addr_recv;
}

extern "C" void Terminate()
{
    // Send "END" singal to terminate DRAMSIM
    sprintf(addr_send, "%016lx", 0xffffffffffffffff);
    sprintf(cycle_send, "%d", 0);
    strcat(strcat(addr_send, " END "), cycle_send);
    SendRqst(addr_send);

    close(mfd);
    close(cfd);
}
