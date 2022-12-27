#include <cstdio>
#include <cstdlib>
#include <utility>
#include <cassert>
#include <vector>
#include <iostream>

using namespace std;

int main(int argc, char **argv)
{
    std::cout << "Hello World!" << endl;

    int mat_sz;
    sscanf(argv[1], "%d", &mat_sz);

    std::cout << "matrix size: ";
    std::cout << mat_sz << endl;

    FILE *output_path = fopen(argv[2], "w");
    FILE *meta_path = fopen(argv[3], "w");

    assert(__builtin_popcount(mat_sz) == 1);

    int log_mat_sz = 0;
    while (mat_sz != (1 << log_mat_sz))
    {
        log_mat_sz++;
        std::cout << "Log matrix size: ";
        std::cout << log_mat_sz << endl;
    }

    // input layer
    int block_number = mat_sz * mat_sz;
    std::cout << "Block Number:";
    std::cout << block_number << endl;
    int block_size;
    std::cout << "Block Size:";
    std::cout << block_size << endl;

    vector<vector<int>> A, B;
    // Create empty vector A, B with size 16 or mat_sz
    A.resize(mat_sz), B.resize(mat_sz);
    // for (int a = 0; a < A.size(); ++a)
    // {
    //     std::cout << "a in A:";
    //     std::cout << a << ' ';
    //     std::cout << "\n";
    // }

    // std::cout << "B:";
    // std::cout << B << endl;

    // Create random matrix A and B size 16x16 or (mat_sz)x(mat_sz)
    for (int i = 0; i < mat_sz; ++i)
    {
        A[i].resize(mat_sz);
        B[i].resize(mat_sz);
        for (int j = 0; j < mat_sz; ++j)
        {
            A[i][j] = rand() % 10, B[i][j] = rand() % 10;
            // std::cout << A[i][j] << endl;
        }
    }

    std::cout << "Matrix A: \n";
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            std::cout << A[i][j];
        }
        std::cout << "\n";
    }

    std::cout << "Matrix B: \n";
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            std::cout << B[i][j];
        }
        std::cout << "\n";
    }

    // input layer
    // write 3 on the first line of the circuit file
    fprintf(output_path, "%d\n", 1 + 1 + 1);
    // write mat_sz*mat_sz*2 on the seconde line of the circuit file
    fprintf(output_path, "%d ", mat_sz * mat_sz * 2);
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            fprintf(output_path, "%d %d %010d %d ", 3, i * mat_sz + j, A[i][j], 0);
            std::cout << i * mat_sz + j;
            std::cout << "\n";
        }
    }
    fprintf(meta_path, "0 0 1 0 0\n");
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            fprintf(output_path, "%d %d %010d %d ", 3, mat_sz * mat_sz + i * mat_sz + j, B[i][j], 0);
        }
    }
    fprintf(output_path, "\n");

    // mult
    fprintf(output_path, "%d ", mat_sz * mat_sz * mat_sz);
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            for (int k = 0; k < mat_sz; ++k)
            {
                int id = i * mat_sz * mat_sz + j * mat_sz + k;
                int a = i * k, b = k * j;
                fprintf(output_path, "%d %d %d %d ", 1, id, a, b);
            }
        }
    }
    fprintf(meta_path, "0 0 1 0 0\n");
    fprintf(output_path, "\n");

    // summation gate
    fprintf(output_path, "%d ", mat_sz * mat_sz);
    for (int i = 0; i < mat_sz; ++i)
    {
        for (int j = 0; j < mat_sz; ++j)
        {
            fprintf(output_path, "5 %d %d %d ", i * mat_sz + j, i * mat_sz * mat_sz + j * mat_sz, i * mat_sz * mat_sz + j * mat_sz + mat_sz);
        }
    }
    fprintf(meta_path, "0 0 1 0 0\n");
    fclose(output_path);
    fclose(meta_path);
    return 0;
}
