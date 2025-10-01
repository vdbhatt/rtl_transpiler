module simple_fsm (
    input wire clk,
    input wire reset,
    input wire start,
    output wire done,
    output wire busy,
    output wire [1:0] state_out
);

    reg /* state_type */ state;
    reg /* state_type */ next_state;

    always @(posedge clk or posedge reset) begin
        if reset == 1'b1 begin
        state <= IDLE;
        end else begin
        state <= next_state;
        end
    end

    always @(*) begin
        case (state)
        IDLE: begin
        if start == 1'b1 begin
        next_state <= RUNNING;
        end else begin
        next_state <= IDLE;
        end
        end
        RUNNING: begin
        next_state <= FINISH;
        end
        FINISH: begin
        next_state <= IDLE;
        end
        default: begin
        next_state <= IDLE;
        end
        endcase
    end
endmodule

