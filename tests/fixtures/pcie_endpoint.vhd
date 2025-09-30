-- PCIe Endpoint Core Interface
entity pcie_endpoint is
    port(
        -- System Interface
        sys_clk_p       : in  std_logic;
        sys_clk_n       : in  std_logic;
        sys_rst_n       : in  std_logic;

        -- User Clock and Reset
        user_clk        : out std_logic;
        user_reset      : out std_logic;
        user_lnk_up     : out std_logic;

        -- PCIe Lanes
        pci_exp_txp     : out std_logic_vector(7 downto 0);
        pci_exp_txn     : out std_logic_vector(7 downto 0);
        pci_exp_rxp     : in  std_logic_vector(7 downto 0);
        pci_exp_rxn     : in  std_logic_vector(7 downto 0);

        -- AXI Stream TX
        s_axis_tx_tdata : in  std_logic_vector(255 downto 0);
        s_axis_tx_tkeep : in  std_logic_vector(31 downto 0);
        s_axis_tx_tlast : in  std_logic;
        s_axis_tx_tvalid: in  std_logic;
        s_axis_tx_tready: out std_logic;
        s_axis_tx_tuser : in  std_logic_vector(3 downto 0);

        -- AXI Stream RX
        m_axis_rx_tdata : out std_logic_vector(255 downto 0);
        m_axis_rx_tkeep : out std_logic_vector(31 downto 0);
        m_axis_rx_tlast : out std_logic;
        m_axis_rx_tvalid: out std_logic;
        m_axis_rx_tready: in  std_logic;
        m_axis_rx_tuser : out std_logic_vector(21 downto 0);

        -- Configuration
        cfg_mgmt_addr   : in  std_logic_vector(18 downto 0);
        cfg_mgmt_write  : in  std_logic;
        cfg_mgmt_write_data: in std_logic_vector(31 downto 0);
        cfg_mgmt_byte_enable: in std_logic_vector(3 downto 0);
        cfg_mgmt_read   : in  std_logic;
        cfg_mgmt_read_data: out std_logic_vector(31 downto 0);
        cfg_mgmt_read_write_done: out std_logic;

        -- Status
        cfg_link_power_state: out std_logic_vector(1 downto 0);
        cfg_err_cor     : out std_logic;
        cfg_err_fatal   : out std_logic;
        cfg_err_nonfatal: out std_logic;
        cfg_local_error : out std_logic
    );
end entity pcie_endpoint;