module spi_master (
    input wire clk,
    input wire rst_n,
    input wire start,
    output wire busy,
    output wire done,
    input wire [7:0] tx_data,
    output wire [7:0] rx_data,
    input wire cpol,
    input wire cpha,
    input wire [7:0] clk_div,
    output wire sclk,
    output wire mosi,
    input wire miso,
    output wire cs_n
);
endmodule

