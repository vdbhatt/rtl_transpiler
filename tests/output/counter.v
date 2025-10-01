module UP_COUNTER (
    input wire clk,
    input wire reset,
    output wire [3:0] counter
);

    reg [3:0] counter_up;

    always @(posedge clk) begin
        if (reset == 1'b1) begin
        counter_up <= 4'h0;
        end else begin
        counter_up <= counter_up + 4'h1;
        end
        end
    end

    assign counter = counter_up;
endmodule

