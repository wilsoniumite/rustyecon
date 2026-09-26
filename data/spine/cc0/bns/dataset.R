################################################################################
# CREATES DATASETS FROM RAW DATA
# called by main_estimation.R
################################################################################

setwd(path_raw)

# Population data
df <- read_excel('MalthusFigures.xlsx', sheet = 'Wages and Population', skip = 1)
df <- df[c('Decade', 'Pop England in millions',
           "Helper's Real Wage (1860s=100)")]
colnames(df) <- c('decade', 'pop_10', 'w_help_05')

# Clark's wage data
df_w <- read_excel('Wages 2014.xlsx', sheet = 'Decadal')
df_w <- df_w[c('Decade', 'Real Farm Wage (1860s=100)',
               'Real Building Laborer Wage (1860s=100)',
               'Real Building Craftsman Wage (1860s=100)')]
colnames(df_w) <- c('decade', 'w_farm_10', 'w_lab_10', 'w_craft_10')
df <- merge(df, df_w, by = 'decade', all = TRUE)
df <- df[c('decade', 'pop_10', 'w_farm_10', 'w_lab_10', 'w_craft_10',
           'w_help_05')]

# Allen's wage data
df_allen <- read_excel('allen07.xlsx', sheet = 'allen07_app2')
df_allen <- df_allen %>%
  mutate(decade = floor(year/10)*10) %>%
  group_by(decade) %>%
  summarize(allen_w = mean(allen_w))
df <- merge(df, df_allen, by = 'decade', all = TRUE)
norm <- df %>% filter(decade == 1770) %>%
  mutate(ratio = w_lab_10 / allen_w) %>%
  pull(ratio)
df <- df %>% mutate(allen_w = allen_w * norm) %>% 
  mutate(allen_w = coalesce(allen_w, w_lab_10))

# Days worked data
df_hw <- read_excel('hw19.xlsx', sheet = 'hw19_tabA2', skip = 1)
df_hw <- df_hw[c('...1', 'Implied income...5', 'Real income...6')]
colnames(df_hw) <- c('decade','ninc_hw', 'inc_hw')
df_hw <- df_hw %>% mutate(decade = as.numeric(substr(decade, 1, 4)))
df_nw <- read_excel('Wages 2014.xlsx', sheet = 'Decadal')
df_nw <- df_nw[c('Decade', 'Farm Laborers, d/day', 'Building Laborers, d/day',
                 'Building Craftsmen, d/day')]
colnames(df_nw) <- c('decade', 'nw_farm', 'nw_lab', 'nw_craft')
df_hw <- merge(df_hw, df_nw, by = 'decade', all = TRUE)
df_hw <- df_hw %>% mutate(d_hw = ninc_hw / nw_farm) %>%
  select(decade, d_hw, inc_hw)
df <- merge(df, df_hw, by = 'decade', all = TRUE)

# Broadberry et al.'s population estimate
df_broad <- read_excel('a-millennium-of-macroeconomic-data-for-the-uk.xlsx',
                       sheet = 'A2. Pop of Eng & GB 1086-1870', skip = 7)
df_broad <- df_broad[c('Year', 'Population of England, millions')]
colnames(df_broad) <- c('year', 'broad_pop_hf')
df_broad <- df_broad %>%
  mutate(decade = floor(year/10)*10) %>%
  group_by(decade) %>% summarize(broad_pop_hf = mean(broad_pop_hf))
df_broad2 <- read_excel('broadberry_etal15.xlsx') %>%
  mutate(decade = floor(year/10)*10)
df_broad <- merge(df_broad, df_broad2, by = 'decade', all = TRUE) %>%
  select(-year) %>%
  mutate(broad_pop_lf = ifelse(decade < 1540, broad_pop_lf, broad_pop_hf))
df <- merge(df, df_broad, by = 'decade', all = TRUE)

# Clark's population index
df_ind <- read_excel('clark07.xlsx')
df_ind <- df_ind[c('Decade', 'Population of sample communities (1310s = 100)',
                   'Number of communities with population estimates',
                   "‘Best’ population estimate from MPL and sample communities (millions)")]
colnames(df_ind) <- c('decade', 'pop_ind', 'pop_nb_com', 'pop_07')
df_ind <- df_ind %>%
  mutate(decade = as.numeric(substr(decade, 1, 4)),
         pop_ind = ifelse(decade > 1520, NA, pop_ind),
         pop_nb_com = ifelse(decade > 1520, NA, pop_nb_com))
            # stop using that data in 1520
df <- merge(df, df_ind, by = 'decade', all = TRUE)

# Efficiency (Clark 2010)
df_clark10 <- read_excel('clark10.xlsx', sheet = 'clark10_t33')
df_clark10 <- df_clark10[c('Decade', 'Efficiency (P_{DE})', 'Efficiency (P_{NDP})')]
colnames(df_clark10) <- c('decade', 'efficiency_DE', 'efficiency_NDP')
df <- merge(df, df_clark10, by = 'decade', all = TRUE)

# Efficiency (Clark 2016)
df_clark16 <- read_excel('GDP-Efficiency 2023.xlsx', sheet= 'Efficiency', skip = 1)
df_clark16 <- df_clark16[c('Year', 'A-DE', 'A-NDP')]
colnames(df_clark16) <- c('year', 'efficiency_DE16', 'efficiency_NDP16')
df_clark16 <- df_clark16 %>% filter(year == floor(year) + .5) %>%
  mutate(decade = floor(year/10)*10) %>% select(decade, efficiency_DE16, efficiency_NDP16)
df <- merge(df, df_clark16, by = 'decade', all = TRUE)

# TFP agriculture (Allen)
df_tfp <- read_excel('allen05.xlsx', col_names = FALSE)
df_tfp <- t(as.matrix(df_tfp[c(1,3),2:ncol(df_tfp)]))
colnames(df_tfp) <- c('decade', 'allen05_TFP_agriculture')
df <- merge(df, df_tfp, by = 'decade', all = TRUE)

# Capital
df_K <- read_excel('feinstein88.xlsx')
df <- merge(df, df_K, by = 'decade', all = TRUE)

# Interest rate
df_r <- read_excel('clark10.xlsx', sheet = 'clark10_t7')
df_r <- df_r %>%
  mutate(interest_rate = ifelse(is.element(decade, 1370:1540), NA, interest_rate))
    # 5% not measured => not used in estimation
df <- merge(df, df_r, by = 'decade', all = TRUE)

# Labor productivity
df_lp1 <- read_excel('HowGrowthBeganFiguresTables.xlsx',
                    sheet = 'Ag L Prod England', range = 'A3:J9')
df_lp1 <- df_lp1[c('...1', 'Total Pop per 100 Ag Pop')]
colnames(df_lp1) <- c('decade', 'wrigley_prod')
df <- merge(df, df_lp1, by = 'decade', all = TRUE)
df_lp2 <- read_excel('HowGrowthBeganFiguresTables.xlsx',
                     sheet = 'Ag L Prod England', range = 'A13:V20')
df_lp2 <- df_lp2[c('...1', 'Ag L Prod')]
colnames(df_lp2) <- c('decade', 'allen_prod')
df <- merge(df, df_lp2, by = 'decade', all = TRUE)

# Wage/rent indices
df_ind2 <- read_excel('England NNI - Clark - 2015.xlsx', sheet = 'Decadal') %>%
  slice(-1) %>% rename(decade = Decade) %>%
  mutate(w_ind = as.numeric(`Male average Wage`) / as.numeric(`Price Index - Domestic Expenditure`),
         rent_ind = as.numeric(`Land rents`) / as.numeric(`Price Index - Domestic Expenditure`)) %>%
  select(decade, w_ind, rent_ind)
df <- merge(df, df_ind2, by = 'decade', all = TRUE)

# Interest rates
df_r2 <- read_excel('Allreturn2005.xls')
df_r2 <- df_r2[c('Decade', 'Average Land Return (%)', 'Rent Charges (%)')]
colnames(df_r2) <- c('decade', 'r_land', 'r_rents')
df <- merge(df, df_r2, by = 'decade', all = TRUE)

# Interest rates
df_sh <- read_excel('clark10.xlsx', sheet = 'clark10_t13')
df_sh <- df_sh[c('Decade', 'Land', 'Capital')]
colnames(df_sh) <- c('decade', 'land_share', 'capital_share')
df <- merge(df, df_sh, by = 'decade', all = TRUE)

# Wrigley population
df <- df %>% mutate(wrigley_pop = ifelse(decade < 1540, NA, pop_10))

# Final file
df <- df %>% filter(is.element(decade, 120:186 * 10))
write.csv(df, file = paste0(path_trans, '/malthus_data.csv'), sep = ',', na = '', row.names = FALSE)
