module fifo (
    input wire clk,
    input wire rst,
    input wire wr_en,
    input wire rd_en,
    input wire [31:0] data_in,
    output wire [31:0] data_out,
    output wire full,
    output wire empty,
    output wire almost_full,
    output wire almost_empty,
    output wire wr_ack,
    output wire valid,
    output wire overflow,
    output wire underflow,
    output wire [7:0] count
);
endmodule

