### SMTP Overview

SMTP (Simple Mail Transfer Protocol) is a communication protocol for mail transfer between servers and clients. It is an application layer protocol built ontop of a TCP transport layer. An AMTP session consists of a series of commands and responses exchanged between the client and server. Some of the basic commands include:

1. HELO/EHLO: Initiate the SMTP session.

2. MAIL FROM: Specify the sender's email address.

3. RCPT TO: Specify the recipient's email address.

4. DATA: Indicate that the client will now send the email content. 

5. QUIT: Terminate the SMTP session.



#### SMTP Command Semantics and Syntax

1. **Extended HELLO (EHLO) or HELLO (HELO)**: These commands are used to identify the SMTP client to the SMTP server. The argument clause contains the fully-qualified domain name of the SMTP client, if one is available. 



Implementing the fuzzing functionality for SMTP will require the capability of generating random SMTP messages and mutating existing messages to explore different edge cases and vulnerabilities. 

#### SMTP Traffic Analysis

**Scenario**: SMTP client sends email to SMTP server. No authentication required on either endpoint.

**Summary**: Client establishes TCP connection with server.

**Packet Capture**: 

1. First 3 packets represent TCP 3-way handshake

2. Server -> Client: 53 bytes
   
   * Response code: 220 - Indicating the service is ready
   
   * Domain of server: ARCH-DSK-01.localdomain
   
   * SMTP Version?: Python SMTP 1.4.4.post2

3. Client -> Server: TCP only (ACK)

4. Client -> Server: 30 bytes
   
   * Message Type: EHLO - Initiates the SMTP session and identifies the client's domain
   
   * Domain of sender: ARCH-LPT-01.localdomain

5. Server -> Client: TCP only (ACK)

6. Server -> Client: 29 bytes
   
   * Response code: 250 - The previous EHLO command was accepted and processed successfully
   
   * Domain of server: ARCH-DSK-01.localdomain

7. Server -> Client: 33 bytes
   
   * Response code: 250
   
   * Max Message Size: 33,554,432
   
   * MIME Encoding: 8-bit

8. Server -> Client: 24 bytes
   
   * Response code: 250
   
   * Supported Extensions: SMTPUTF8
   
   * "250 HELP" - Server provides a HELP command with additional information on supported commands
