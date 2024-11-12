use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug)]
pub struct AnnuityDetails {
    pub contract_type: String,
    pub contract_role: String,
    pub contract_identifier: String,
    pub nominal_interest_rate: Decimal,
    pub currency: String,
    pub initial_exchange_date: u64,
    pub maturity_date: u64,
    pub notional_principal: Decimal,
    pub annuity_position: String,
    pub price: Decimal,
    pub amount: Decimal,
    pub maturity_days_left: i64,
    pub annual_payout: Decimal,
    pub last_payout_epoch: u64,
}

#[blueprint]
mod annuity {

    struct Annuity {
        contract_type: String,
        contract_role: String,
        contract_identifier: String,
        nominal_interest_rate: Decimal,
        currency: String,
        initial_exchange_date: u64,
        maturity_date: u64,
        notional_principal: Decimal,
        annuity_position: String,
        annuities: Vault,
        collected_xrd: Vault,
        price: Decimal,
        annual_payout: Decimal,
        last_payout_epoch: u64,
    }

    impl Annuity {
        pub fn instantiate_annuity(
            contract_type: String,
            contract_role: String,
            contract_identifier: String,
            nominal_interest_rate: Decimal,
            currency: String,
            initial_exchange_date: u64,
            maturity_date: u64,
            notional_principal: Decimal,
            annuity_position: String,
            price: Decimal,
            number_of_annuities_to_mint: Decimal,
        ) -> Global<Annuity> {
            let bucket_of_annuities: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Annuity", locked;
                        "symbol" => "ANN", locked;
                        "description" => "A Fixed Rate Annuity", locked;
                    }
                ))
                .mint_initial_supply(number_of_annuities_to_mint)
                .into();

            let annual_payout = notional_principal / Decimal::from(5);

            Self {
                contract_type,
                contract_role,
                contract_identifier,
                nominal_interest_rate,
                currency,
                initial_exchange_date,
                maturity_date,
                notional_principal,
                annuity_position,
                annuities: Vault::with_bucket(bucket_of_annuities),
                collected_xrd: Vault::new(XRD),
                price,
                annual_payout,
                last_payout_epoch: initial_exchange_date,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn purchase_annuity(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
            let our_share = payment.take(self.price);
            self.collected_xrd.put(our_share);
            (self.annuities.take(1), payment)
        }

        pub fn get_current_epoch(&self) -> u64 {
            Runtime::current_epoch().number()
        }

        pub fn get_current_time(&self) -> i64 {
            let current_time = Clock::current_time_rounded_to_seconds();
            println!("Current time: {:?}", current_time);
            println!(
                "Seconds since unix epoch: {}",
                current_time.seconds_since_unix_epoch
            );

            current_time.seconds_since_unix_epoch
        }

        pub fn get_current_timestamp(&self) -> i64 {
            Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch
        }

        pub fn format_instant(&self, seconds : i64) -> String {
            // let seconds = instant.seconds_since_unix_epoch;
            let minutes = seconds / 60;
            let hours = minutes / 60;
            let days = hours / 24;

            let remaining_seconds = seconds % 60;
            let remaining_minutes = minutes % 60;
            let remaining_hours = hours % 24;

            format!(
                "{} days, {} hours, {} minutes, {} seconds",
                days, remaining_hours, remaining_minutes, remaining_seconds 
            )
        }

        pub fn claim_annual_payout(&mut self, annuity_token: Bucket) {
            assert!(
                annuity_token.amount() == Decimal::one(),
                "You can only claim for one annuity (ANN) at a time."
            );

            assert!(
                annuity_token.resource_address() == self.annuities.resource_address(),
                "Invalid annuity resource address. Please make sure the token is an Annuity (ANN)."
            );
        }

        pub fn get_annuity_details(&self) -> AnnuityDetails {
            let current_epoch = Runtime::current_epoch().number();
            let seconds_in_day = 24 * 60 * 60;
            let days_left = (self.maturity_date as i64 - current_epoch as i64) / seconds_in_day;

            AnnuityDetails {
                contract_type: self.contract_type.clone(),
                contract_role: self.contract_role.clone(),
                contract_identifier: self.contract_identifier.clone(),
                nominal_interest_rate: self.nominal_interest_rate,
                currency: self.currency.clone(),
                initial_exchange_date: self.initial_exchange_date,
                maturity_date: self.maturity_date,
                notional_principal: self.notional_principal,
                annuity_position: self.annuity_position.clone(),
                price: self.price,
                amount: self.annuities.amount(),
                maturity_days_left: days_left,
                annual_payout: self.annual_payout,
                last_payout_epoch: self.last_payout_epoch,
            }
        }
    }
}

// Example commands to call functions
// resim call-function package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92 Annuity instantiate_annuity ANN issuer CONTRACT1234 0.05 XRD 1719321600 1877088000 1000 long 1000 10
// component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2
// resim show component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2
// resim show account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma

// resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 get_annuity_details
// resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 purchase_annuity resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:1000
// resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 claim_annual_payout resource_sim1t4h3kupr5l95w6ufpuysl0afun0gfzzw7ltmk7y68ks5ekqh4cpx9w:1
// resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 check_time_until_next_payout

// package_tdx_2_1pklk5h22xd2exahfhckcgay7ew8ggj54wctwc6w5yrxyqm65yeu3r6
