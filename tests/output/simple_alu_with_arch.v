module simple_alu (
    input wire [7:0] a,
    input wire [7:0] b,
    input wire [1:0] opcode,
    output wire [7:0] result,
    output wire zero
);

    reg [7:0] result_temp;

    always @(*) begin
        case (opcode)
        2'b00: begin
        result_temp <= a + b;
        end
        2'b01: begin
        result_temp <= a - b;
        end
        2'b10: begin
        result_temp <= a & b;
        end
        2'b11: begin
        result_temp <= a | b;
        end
        default: begin
        result_temp <= 8'b0;
        end
        endcase
    end

    assign result = result_temp;
endmodule

