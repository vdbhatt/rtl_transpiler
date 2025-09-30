-- Simple 8-bit counter with architecture
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity counter_with_arch is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        enable : in  std_logic;
        count  : out std_logic_vector(7 downto 0)
    );
end entity counter_with_arch;

architecture rtl of counter_with_arch is
    signal count_reg : unsigned(7 downto 0);
begin

    -- Counter process
    process(clk, reset)
    begin
        if reset = '1' then
            count_reg <= (others => '0');
        elsif rising_edge(clk) then
            if enable = '1' then
                count_reg <= count_reg + 1;
            end if;
        end if;
    end process;

    -- Output assignment
    count <= std_logic_vector(count_reg);

end architecture rtl;