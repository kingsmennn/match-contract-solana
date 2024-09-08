
````markdown
# Match Solana Contract

This repository contains a Solana smart contract for a decentralized marketplace built with Anchor. The contract allows users to create buyer/seller profiles, create stores and requests, and submit or accept offers.

## Prerequisites

To work with this project, you'll need the following tools installed:

- [Node.js](https://nodejs.org/en/) (v14 or higher)
- [Yarn](https://yarnpkg.com/getting-started/install) (v1.x)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html)
- A local Solana cluster
## Setup

### 1. Install Dependencies

Clone this repository and install all required dependencies using Yarn:

```bash
git clone https://github.com/kingsmennn/match-contract-solana
cd match-solana-contract
yarn install
```
````

### 2. Configure Local Environment

Anchor uses `anchor.toml` for configurations. You can customize this file to specify the cluster you want to use (localnet or devnet) and the program ID.

```toml
[provider]
cluster = "localnet"
wallet = "/path/to/your/solana/wallet.json"
```

### 3. Build and Deploy the Contract

To build the smart contract and deploy it to your local Solana cluster:

```bash
anchor build
```

### 4. Running Tests

To run the tests, you'll need to first start your local Solana cluster and then execute the test cases using the following commands:

```bash
anchor test
```

### Program Overview

The contract consists of several key features:

- **User Account Management**: Buyers and sellers can create accounts with their profile details such as username, phone number, and geolocation.
- **Store Management**: Sellers can create stores by providing store details.
- **Request Management**: Buyers can create requests for specific products or services.
- **Offer Management**: Sellers can submit offers to requests, and buyers can accept or reject those offers.

### Payload Structure

- **Buyer/Seller Profiles**: Both buyers and sellers are stored on-chain with attributes like `username`, `phone`, and location (`latitude`, `longitude`).
- **Store**: Each store contains a `name`, `description`, `phone`, and location (`lat`, `long`).
- **Request**: A buyer request consists of a `name`, `description`, images, and location.
- **Offer**: An offer includes the price, store name, and images associated with the offer.

### Key Constants

The following constants are imported from the `utils` module to standardize on-chain data handling:

- `LOCATION_DECIMALS`: Defines the precision for location data.
- `USER_TAG`, `STORE_TAG`, `REQUEST_TAG`, `OFFER_TAG`: Used to generate PDAs for various entities.
- `USER_COUNTER`, `STORE_COUNTER`, `REQUEST_COUNTER`, `OFFER_COUNTER`: Used for counting created entities.

### Test Cases

The test suite covers the following scenarios:

1. **Creating Users**: Tests the creation of both buyer and seller accounts.
2. **Updating User Details**: Verifies that user profiles can be updated successfully.
3. **Store Creation**: Allows sellers to create stores.
4. **Request Creation**: Allows buyers to create requests for specific products/services.
5. **Offer Creation**: Enables sellers to make offers in response to buyer requests.
6. **Offer Acceptance**: Ensures that buyers can accept an offer, and all other offers are updated accordingly.

### Development Workflow

1. **Initialize Counters**: The contract initializes counters for users, stores, requests, and offers.
2. **Create User**: Users (buyers and sellers) are created with profile information.
3. **Create Store**: Sellers can create a store with details such as name and description.
4. **Create Request**: Buyers can submit requests for specific items.
5. **Create Offer**: Sellers can respond to requests with offers.
6. **Accept Offer**: Buyers can accept an offer, finalizing the transaction.


## License

This project is licensed under the MIT License.

