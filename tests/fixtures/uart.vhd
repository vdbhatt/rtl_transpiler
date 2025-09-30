-- UART Transmitter and Receiver
entity uart is
    port(
        -- Clock and Reset
        clk             : in  std_logic;
        reset_n         : in  std_logic;

        -- Transmit Interface
        tx_data         : in  std_logic_vector(7 downto 0);
        tx_start        : in  std_logic;
        tx_busy         : out std_logic;
        tx_done         : out std_logic;
        tx_out          : out std_logic;

        -- Receive Interface
        rx_in           : in  std_logic;
        rx_data         : out std_logic_vector(7 downto 0);
        rx_valid        : out std_logic;
        rx_error        : out std_logic;

        -- Configuration
        baud_rate_div   : in  std_logic_vector(15 downto 0);
        parity_enable   : in  std_logic;
        parity_odd      : in  std_logic;
        stop_bits       : in  std_logic_vector(1 downto 0)
    );
end entity uart;