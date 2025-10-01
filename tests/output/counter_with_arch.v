module counter_with_arch (
    input wire clk,
    input wire reset,
    input wire enable,
    output wire [7:0] count
);

    reg [7:0] count_reg;

    always @(posedge clk or posedge reset) begin
        if reset == 1'b1 begin
        count_reg <= 8'b0;
        end else begin
        if enable == 1'b1 begin
        count_reg <= count_reg + 1;
        end
        end
    end

    assign count = count_reg;
endmodule

