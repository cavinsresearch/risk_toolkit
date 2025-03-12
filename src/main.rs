mod analysis;
mod data;

use analysis::{
    factor_config::{create_default_factor_groups, create_pca_factor_groups},
    factor_model::ThematicFactorModel,
    pca::PCA,
    risk_attribution::RiskAttributor,
};
use data::loader::DataLoader;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get data file path from command line or use default
    let data_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "data/prices.csv".to_string());

    println!("Loading data from: {}", data_path);
    let (_market_data, returns) = DataLoader::load_and_calculate_returns(&data_path)?;
    let tickers = DataLoader::get_tickers(&data_path)?;

    println!(
        "\nData shape: {} time periods × {} assets",
        returns.nrows(),
        returns.ncols()
    );

    // First, run PCA analysis
    println!("\n=== Statistical Factor Analysis (PCA) ===");
    let pca = PCA::new(Some(3)); // Keep top 3 factors
    let mut pca_result = pca.fit_transform(returns.view())?;

    // Print explained variance ratios
    println!("\nExplained variance ratios:");
    let mut cumulative = 0.0;
    pca_result
        .explained_variance_ratio
        .iter()
        .enumerate()
        .for_each(|(i, ratio)| {
            cumulative += ratio;
            println!(
                "Factor {}: {:.4} (cumulative: {:.4})",
                i + 1,
                ratio,
                cumulative
            );
        });

    // Set asset list for PCA factors
    for group in pca_result.factor_model.get_factor_groups_mut() {
        group.assets = tickers.clone();
    }

    // Compute risk attribution for PCA factors
    println!("\n=== PCA Risk Attribution Analysis ===");
    let risk_attributor = RiskAttributor::new(pca_result.factor_model, 252); // 1 year lookback
    let attributions = risk_attributor.compute_portfolio_risk_attribution(
        returns.view(),
        &tickers,
        None, // No portfolio weights
    )?;

    // Print PCA risk attribution for each asset
    println!("\nPCA Risk Attribution by Asset:");
    for (i, attribution) in attributions.iter().enumerate() {
        if i < tickers.len() {
            println!("\n{}:", tickers[i]);
            println!("Total Risk: {:.2}%", attribution.total_risk * 100.0);
            println!("R-squared: {:.2}%", attribution.r_squared * 100.0);
            println!("Factor Contributions:");
            let mut contributions: Vec<_> = attribution.factor_contributions.iter().collect();
            contributions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            for (factor, contribution) in contributions {
                println!("  {}: {:.2}%", factor, contribution * 100.0);
            }
        }
    }

    // Now, let's do thematic factor analysis
    println!("\n=== Thematic Factor Analysis ===");
    let factor_groups = create_default_factor_groups();
    println!("\nDefined {} thematic factors:", factor_groups.len());
    for group in &factor_groups {
        println!("\n{}: {}", group.name, group.description);
        println!("Assets: {}", group.assets.join(", "));
    }

    let factor_model = ThematicFactorModel::new(factor_groups);
    let factor_returns = factor_model.compute_factor_returns(returns.view(), &tickers)?;

    println!(
        "\nThematic Factor Returns Shape: {} periods × {} factors",
        factor_returns.nrows(),
        factor_returns.ncols()
    );

    // Compute risk attribution for thematic factors
    println!("\n=== Thematic Risk Attribution Analysis ===");
    let risk_attributor = RiskAttributor::new(factor_model, 252); // 1 year lookback
    let attributions = risk_attributor.compute_portfolio_risk_attribution(
        returns.view(),
        &tickers,
        None, // No portfolio weights
    )?;

    // Print thematic risk attribution for each asset
    println!("\nThematic Risk Attribution by Asset:");
    for (i, attribution) in attributions.iter().enumerate() {
        if i < tickers.len() {
            println!("\n{}:", tickers[i]);
            println!("Total Risk: {:.2}%", attribution.total_risk * 100.0);
            println!("R-squared: {:.2}%", attribution.r_squared * 100.0);
            println!("Factor Contributions:");
            let mut contributions: Vec<_> = attribution.factor_contributions.iter().collect();
            contributions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            for (factor, contribution) in contributions {
                println!("  {}: {:.2}%", factor, contribution * 100.0);
            }
        }
    }

    Ok(())
}
