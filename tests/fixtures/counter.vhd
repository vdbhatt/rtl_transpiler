-- Simple 8-bit counter entity
entity counter is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        enable : in  std_logic;
        count  : out std_logic_vector(7 downto 0)
    );
end entity counter;