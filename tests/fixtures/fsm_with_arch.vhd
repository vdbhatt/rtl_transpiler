-- Simple FSM with architecture
library ieee;
use ieee.std_logic_1164.all;

entity simple_fsm is
    port(
        clk    : in  std_logic;
        reset  : in  std_logic;
        start  : in  std_logic;
        done   : out std_logic;
        busy   : out std_logic;
        state_out : out std_logic_vector(1 downto 0)
    );
end entity simple_fsm;

architecture rtl of simple_fsm is
    type state_type is (IDLE, RUNNING, FINISH);
    signal state, next_state : state_type;
begin

    -- State register
    process(clk, reset)
    begin
        if reset = '1' then
            state <= IDLE;
        elsif rising_edge(clk) then
            state <= next_state;
        end if;
    end process;

    -- Next state logic
    process(state, start)
    begin
        case state is
            when IDLE =>
                if start = '1' then
                    next_state <= RUNNING;
                else
                    next_state <= IDLE;
                end if;

            when RUNNING =>
                next_state <= FINISH;

            when FINISH =>
                next_state <= IDLE;

            when others =>
                next_state <= IDLE;
        end case;
    end process;

    -- Output logic
    done <= '1' when state = FINISH else '0';
    busy <= '1' when state = RUNNING else '0';

    with state select
        state_out <= "00" when IDLE,
                     "01" when RUNNING,
                     "10" when FINISH,
                     "11" when others;

end architecture rtl;