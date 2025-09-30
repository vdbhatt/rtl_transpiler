-- Synchronous FIFO with depth and width parameters
entity fifo is
    port(
        clk         : in  std_logic;
        rst         : in  std_logic;
        wr_en       : in  std_logic;
        rd_en       : in  std_logic;
        data_in     : in  std_logic_vector(31 downto 0);
        data_out    : out std_logic_vector(31 downto 0);
        full        : out std_logic;
        empty       : out std_logic;
        almost_full : out std_logic;
        almost_empty: out std_logic;
        wr_ack      : out std_logic;
        valid       : out std_logic;
        overflow    : out std_logic;
        underflow   : out std_logic;
        count       : out std_logic_vector(7 downto 0)
    );
end entity fifo;