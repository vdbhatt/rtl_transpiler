module uart (
    input wire clk,
    input wire reset_n,
    input wire [7:0] tx_data,
    input wire tx_start,
    output wire tx_busy,
    output wire tx_done,
    output wire tx_out,
    input wire rx_in,
    output wire [7:0] rx_data,
    output wire rx_valid,
    output wire rx_error,
    input wire [15:0] baud_rate_div,
    input wire parity_enable,
    input wire parity_odd,
    input wire [1:0] stop_bits
);
endmodule

