-- AXI4 Crossbar Switch
entity axi_crossbar is
    port(
        -- Global Signals
        aclk            : in  std_logic;
        aresetn         : in  std_logic;

        -- Master 0 Interface
        m0_awid         : in  std_logic_vector(7 downto 0);
        m0_awaddr       : in  std_logic_vector(31 downto 0);
        m0_awlen        : in  std_logic_vector(7 downto 0);
        m0_awsize       : in  std_logic_vector(2 downto 0);
        m0_awburst      : in  std_logic_vector(1 downto 0);
        m0_awvalid      : in  std_logic;
        m0_awready      : out std_logic;
        m0_wdata        : in  std_logic_vector(63 downto 0);
        m0_wstrb        : in  std_logic_vector(7 downto 0);
        m0_wlast        : in  std_logic;
        m0_wvalid       : in  std_logic;
        m0_wready       : out std_logic;
        m0_bid          : out std_logic_vector(7 downto 0);
        m0_bresp        : out std_logic_vector(1 downto 0);
        m0_bvalid       : out std_logic;
        m0_bready       : in  std_logic;
        m0_arid         : in  std_logic_vector(7 downto 0);
        m0_araddr       : in  std_logic_vector(31 downto 0);
        m0_arlen        : in  std_logic_vector(7 downto 0);
        m0_arsize       : in  std_logic_vector(2 downto 0);
        m0_arburst      : in  std_logic_vector(1 downto 0);
        m0_arvalid      : in  std_logic;
        m0_arready      : out std_logic;
        m0_rid          : out std_logic_vector(7 downto 0);
        m0_rdata        : out std_logic_vector(63 downto 0);
        m0_rresp        : out std_logic_vector(1 downto 0);
        m0_rlast        : out std_logic;
        m0_rvalid       : out std_logic;
        m0_rready       : in  std_logic;

        -- Slave 0 Interface
        s0_awid         : out std_logic_vector(7 downto 0);
        s0_awaddr       : out std_logic_vector(31 downto 0);
        s0_awlen        : out std_logic_vector(7 downto 0);
        s0_awsize       : out std_logic_vector(2 downto 0);
        s0_awburst      : out std_logic_vector(1 downto 0);
        s0_awvalid      : out std_logic;
        s0_awready      : in  std_logic;
        s0_wdata        : out std_logic_vector(63 downto 0);
        s0_wstrb        : out std_logic_vector(7 downto 0);
        s0_wlast        : out std_logic;
        s0_wvalid       : out std_logic;
        s0_wready       : in  std_logic;
        s0_bid          : in  std_logic_vector(7 downto 0);
        s0_bresp        : in  std_logic_vector(1 downto 0);
        s0_bvalid       : in  std_logic;
        s0_bready       : out std_logic;
        s0_arid         : out std_logic_vector(7 downto 0);
        s0_araddr       : out std_logic_vector(31 downto 0);
        s0_arlen        : out std_logic_vector(7 downto 0);
        s0_arsize       : out std_logic_vector(2 downto 0);
        s0_arburst      : out std_logic_vector(1 downto 0);
        s0_arvalid      : out std_logic;
        s0_arready      : in  std_logic;
        s0_rid          : in  std_logic_vector(7 downto 0);
        s0_rdata        : in  std_logic_vector(63 downto 0);
        s0_rresp        : in  std_logic_vector(1 downto 0);
        s0_rlast        : in  std_logic;
        s0_rvalid       : in  std_logic;
        s0_rready       : out std_logic
    );
end entity axi_crossbar;