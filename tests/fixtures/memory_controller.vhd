-- DDR Memory Controller Interface
entity memory_controller is
    port(
        -- System
        clk_sys         : in  std_logic;
        clk_mem         : in  std_logic;
        rst_n           : in  std_logic;

        -- Command Interface
        cmd_valid       : in  std_logic;
        cmd_ready       : out std_logic;
        cmd_addr        : in  std_logic_vector(31 downto 0);
        cmd_write       : in  std_logic;
        cmd_burst_len   : in  std_logic_vector(7 downto 0);

        -- Write Data Interface
        wr_data         : in  std_logic_vector(127 downto 0);
        wr_valid        : in  std_logic;
        wr_ready        : out std_logic;
        wr_strobe       : in  std_logic_vector(15 downto 0);

        -- Read Data Interface
        rd_data         : out std_logic_vector(127 downto 0);
        rd_valid        : out std_logic;
        rd_ready        : in  std_logic;

        -- Status
        init_done       : out std_logic;
        calibration_done: out std_logic;
        error           : out std_logic;

        -- DDR3 Interface
        ddr_ck_p        : out std_logic;
        ddr_ck_n        : out std_logic;
        ddr_cke         : out std_logic;
        ddr_cs_n        : out std_logic;
        ddr_ras_n       : out std_logic;
        ddr_cas_n       : out std_logic;
        ddr_we_n        : out std_logic;
        ddr_ba          : out std_logic_vector(2 downto 0);
        ddr_addr        : out std_logic_vector(14 downto 0);
        ddr_dq          : inout std_logic_vector(31 downto 0);
        ddr_dqs_p       : inout std_logic_vector(3 downto 0);
        ddr_dqs_n       : inout std_logic_vector(3 downto 0);
        ddr_dm          : out std_logic_vector(3 downto 0);
        ddr_odt         : out std_logic
    );
end entity memory_controller;