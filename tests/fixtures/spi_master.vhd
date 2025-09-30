-- SPI Master Controller
entity spi_master is
    port(
        -- System Interface
        clk             : in  std_logic;
        rst_n           : in  std_logic;

        -- Control Interface
        start           : in  std_logic;
        busy            : out std_logic;
        done            : out std_logic;

        -- Data Interface
        tx_data         : in  std_logic_vector(7 downto 0);
        rx_data         : out std_logic_vector(7 downto 0);

        -- Configuration
        cpol            : in  std_logic;
        cpha            : in  std_logic;
        clk_div         : in  std_logic_vector(7 downto 0);

        -- SPI Interface
        sclk            : out std_logic;
        mosi            : out std_logic;
        miso            : in  std_logic;
        cs_n            : out std_logic
    );
end entity spi_master;