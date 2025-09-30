-- Simple ALU with architecture
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity simple_alu is
    port(
        a      : in  std_logic_vector(7 downto 0);
        b      : in  std_logic_vector(7 downto 0);
        opcode : in  std_logic_vector(1 downto 0);
        result : out std_logic_vector(7 downto 0);
        zero   : out std_logic
    );
end entity simple_alu;

architecture behavioral of simple_alu is
    signal result_temp : std_logic_vector(7 downto 0);
begin

    -- ALU operation
    process(a, b, opcode)
    begin
        case opcode is
            when "00" =>
                result_temp <= std_logic_vector(unsigned(a) + unsigned(b));
            when "01" =>
                result_temp <= std_logic_vector(unsigned(a) - unsigned(b));
            when "10" =>
                result_temp <= a and b;
            when "11" =>
                result_temp <= a or b;
            when others =>
                result_temp <= (others => '0');
        end case;
    end process;

    -- Output assignments
    result <= result_temp;
    zero <= '1' when result_temp = "00000000" else '0';

end architecture behavioral;