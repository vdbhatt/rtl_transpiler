module spi_master (
    input wire rst_n,
    output wire busy,
    output wire done,
    output wire [7:0] rx_data,
    input wire cpha,
    input wire [7:0] clk_div,
    output wire mosi,
    input wire miso,
    output wire cs_n
);
endmodule

