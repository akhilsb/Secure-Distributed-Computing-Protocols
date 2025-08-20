# Secure Distributed Computing Protocols

This repository implements a collection of secure distributed computing protocols that serve as building blocks for larger distributed systems. The protocols are designed to provide security guarantees in adversarial environments. 
However, this code has been written as a research prototype and has not been vetted for security. 
Therefore, this repository can contain serious security vulnerabilities. 
Use at your own risk.

## Repository Structure

### Core Protocol Modules
#### **Broadcast Protocols** ([`broadcast/`](broadcast/))
- **CTRBC (Cachin-Tessaro's Reliable Broadcast Protocol)** ([`broadcast/ctrbc/`](broadcast/ctrbc/)) - Cachin-Tessaro's Reliable broadcast protocol based on the protocol in `CT05`. 

- **ECC-RBC (Error-Correcting Code Reliable Broadcast)** ([`broadcast/ecc_rbc/`](broadcast/ecc_rbc/)) - Reliable broadcast using Reed-Solomon error-correcting codes in `NDD+22`.


#### **Dissemination Protocols** ([`dissemination/`](dissemination/))
- **ASKS (Asynchronous Secret Key Sharing)/ AwVSS (Asynchronous weak Verifiable Secret Sharing)** ([`dissemination/asks/`](dissemination/asks/)) - ASKS/AwVSS protocol in the `DDL+24,BBB+24`. 

- **AVID (Asynchronous Verifiable Information Dispersal)** ([`dissemination/avid/`](dissemination/avid/)) - Verifiable information dispersal protocols based on DispersedLedger `SPA+22`. 


#### **Consensus Protocols** ([`consensus/`](consensus/))

- **ACS (Asynchronous Common Subset)** ([`consensus/acs/`](consensus/acs/)) - Implements asynchronous common subset consensus protocol in the `DDL+24`
- **Binary Byzantine Agreement** ([`consensus/binary_ba/`](consensus/binary_ba/)) - Asynchronous Binary BA in `IBY22`.
- **FIN-MVBA (Finite Multi-Valued Byzantine Agreement)** ([`consensus/fin_mvba/`](consensus/fin_mvba/)) - Asynchronous Multi-valued Byzantine agreement protocol in FIN (`SWZ23`).
- **IBFT (Istanbul Byzantine Fault Tolerance)** ([`consensus/ibft/`](consensus/ibft/)) - PBFT-style Leader-based consensus protocol only using Message Authentication Codes in `Hen20`. 
- **RA (Reliable Agreement)** ([`consensus/ra/`](consensus/ra/)) - Reliable agreement protocol in `DDL+24`

## Building and Usage

This is a Rust project using Cargo. The compatibility between dependencies has been tested for Rust version `1.83.0`. To build all components:

```bash
cargo build --release
```
Run the following sequence of steps to start a protocol. 

1. **Generate Configuration Files**: This step generates the necessary configuration files for an $n$ party distributed system. 
```
mkdir testdata/
./target/release/genconfig --base_port 15000 --client_base_port 19000 --client_run_port 19500 --NumNodes 4 --blocksize 100 --delay 100 --target testdata/ --local true
```
These instructions generate configuration files for $n=4$ parties. Party $i$ runs on port `15000+i`, listens to requests on port `19000+i`, and syncs with a global synchronizer (this part is optional) on port `19500`. Please ensure the directory has been created to run this command. 

2. **Create channels and invoke protocol**: The following snippet of code illustrates a basic composition of distributed protocols. 
```rust
pub async fn spawn(config: Node)-> (anyhow::Result<oneshot::Sender<()>>, Vec<Result<oneshot::Sender<()>>>){
    // ctrbc_req_send_channel: Request sending channel, request receiving channel. The sending channel can be used to issue message requests to the RBC module. 
    // ctrbc_req_recv_channel: Request receiving channel - passed as an argument. The RBC module listens to this channel. 
    let (ctrbc_req_send_channel, ctrbc_req_recv_channel) = channel(10000);
    
    // ctrbc_out_send_channel: Output sending channel - passed as an argument. The RBC module sends outputs on this channel. 
    // ctrbc_out_recv_channel: Output receiving channel. We poll this channel to get outputs from RBC module.
    let (ctrbc_out_send_channel, mut ctrbc_out_recv_channel) = channel(10000);

    let mut statuses = Vec::new();

    let _rbc_serv_status = ctrbc::Context::spawn(
        config,
        ctrbc_req_recv_channel, 
        ctrbc_out_send_channel, 
        false
    );

    statuses.push(_rbc_serv_status);
    
    let _resp = ctrbc_req_send_channel.send(Vec::new()).await.unwrap();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = ctrbc_out_recv_channel.recv() => {
                    // Execute handling logic for the received message from the channel
                    log::debug!("Received message from CTRBC channel {:?}", msg);
                    // self.process_ctrbc_event(ctrbc_msg.1, ctrbc_msg.0, ctrbc_msg.2).await;
                }
            }
        }
    });
    let (exit_tx, _exit_rx) = oneshot::channel();
    (Ok(exit_tx), vec![])
}
```
Protocols utilize `tokio` asynchronous channels or queues to receive requests and send outputs.
Each protocol takes two `tokio` channels as input: a receiver channel from which it receives requests (`req_recv` channel), and a sender channel to which it can send outputs (`out_send` channel). 
Each protocol's invocation takes these channels as arguments. 
A prominent example of protocol composition is in `consensus/acs`. 
This folder implements an Asynchronous Common Subset (ACS) protocol from Reliable Broadcast (CTRBC), Secret Key Sharing (ASKS), and Reliable Agreement (RA). 

3. **Build code and run parties**: After compiling the code, run $n=4$ parties to start the protocol. Each party waits until it establishes a tcp channel with **all** parties. 
The `scripts/test.sh` script can also be used to start all four parties locally. 


## Key Features

### Byzantine Fault Tolerance
All protocols are designed to handle Byzantine faults, where up to `t` out of `n` nodes can behave arbitrarily (where typically `n ≥ 3t + 1`).

### Asynchronous Operation
Most protocols operate in asynchronous network models, making no assumptions about message delivery times or clock synchronization.

### Modular Design
Each protocol is implemented as a separate module with well-defined interfaces, allowing them to be composed into larger systems.

### Network Abstraction
The implementation includes a robust networking layer with:
- TCP-based reliable communication
- Message acknowledgments
- Automatic connection management


## Applications

These protocols serve as building blocks for:
- Distributed ledgers and blockchains
- Secure asynchronous multi-party computation protocols
- Byzantine fault-tolerant state machine replication

## Research Context

This implementation is part of ongoing research in secure distributed computing, focusing on practical implementations of theoretically sound protocols that can handle adversarial conditions in distributed systems.

### Supporting Infrastructure

#### **Cryptographic Primitives** ([`crypto/`](crypto/))
- SHA256 Hash function, and Merkle trees based on Hardware-accelerated Hash based on AES
- Symmetric encryption (AES-based)
- Cryptographic utilities and random number generation

#### **Configuration Management** ([`config/`](config/))
- Network configuration and node setup
- Protocol parameter management

#### **Type Definitions** ([`types/`](types/))
- Common data structures and type definitions
- Replica identifiers and protocol messages

#### **Utilities** ([`util/`](util/))
- Helper functions and common utilities
- Networking abstractions

#### **Tools** ([`tools/`](tools/))
- **genconfig** - Configuration generation utility

## References
```
Cachin, Christian, and Stefano Tessaro. "Asynchronous verifiable information dispersal." 24th IEEE Symposium on Reliable Distributed Systems (SRDS'05). IEEE, 2005.

Alhaddad, Nicolas, Sourav Das, Sisi Duan, Ling Ren, Mayank Varia, Zhuolun Xiang, and Haibin Zhang. "Balanced byzantine reliable broadcast with near-optimal communication and improved computation." In Proceedings of the 2022 ACM Symposium on Principles of Distributed Computing, pp. 399-417. 2022.

Das, Sourav, Sisi Duan, Shengqi Liu, Atsuki Momose, Ling Ren, and Victor Shoup. "Asynchronous consensus without trusted setup or public-key cryptography." In Proceedings of the 2024 on ACM SIGSAC Conference on Computer and Communications Security, pp. 3242-3256. 2024.

Bandarupalli, Akhil, Adithya Bhat, Saurabh Bagchi, Aniket Kate, and Michael K. Reiter. "Random beacons in monte carlo: Efficient asynchronous random beacon without threshold cryptography." In Proceedings of the 2024 on ACM SIGSAC Conference on Computer and Communications Security, pp. 2621-2635. 2024.

Yang, Lei, Seo Jin Park, Mohammad Alizadeh, Sreeram Kannan, and David Tse. "{DispersedLedger}:{High-Throughput} byzantine consensus on variable bandwidth networks." In 19th USENIX Symposium on Networked Systems Design and Implementation (NSDI 22), pp. 493-512. 2022.

Abraham, Ittai, Naama Ben-David, and Sravya Yandamuri. "Efficient and adaptively secure asynchronous binary agreement via binding crusader agreement." In Proceedings of the 2022 ACM Symposium on Principles of Distributed Computing, pp. 381-391. 2022.

Duan, Sisi, Xin Wang, and Haibin Zhang. "Fin: Practical signature-free asynchronous common subset in constant time." In Proceedings of the 2023 ACM SIGSAC Conference on Computer and Communications Security, pp. 815-829. 2023.

Moniz, Henrique. "The Istanbul BFT consensus algorithm." arXiv preprint arXiv:2002.03613 (2020).


```