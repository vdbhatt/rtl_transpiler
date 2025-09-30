module uart (
    input wire reset_n,
    input wire tx_start,
    output wire tx_busy,
    output wire tx_done,
    output wire tx_out,
    output wire [7:0] rx_data,
    output wire rx_valid,
    output wire rx_error,
    input wire parity_enable,
    input wire parity_odd,
    input wire [1:0] stop_bits
);
endmodule

