-- Simple ALU entity
entity alu is
    port(
        a      : in  std_logic_vector(15 downto 0);
        b      : in  std_logic_vector(15 downto 0);
        opcode : in  std_logic_vector(2 downto 0);
        result : out std_logic_vector(15 downto 0);
        zero   : out std_logic;
        carry  : out std_logic
    );
end entity alu;